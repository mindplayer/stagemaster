mod stage_mixed_support;
use serde_json::json;
use stage_mixed_support::*;
use stagemaster_project::Document;

#[test]
fn mixed_move_deduplicates_explicit_and_attached_lights_preserving_all_other_data() {
    let (mut doc, targets, _) = setup();
    let before = root(&doc);
    translate(&mut doc, &targets, ["1.25", "0", "-0.25"]).unwrap();
    let after = root(&doc);
    let mut expected = before["stage"].clone();
    for i in 0..2 {
        let p = &mut expected["placements"][i]["positionMeters"];
        p["x"] = ["1.25", "2.25"][i].into();
        p["z"] = ["3.75", "4.75"][i].into();
    }
    let shapes = expected["constructions"].as_array_mut().unwrap();
    shapes[0]["shape"]["positionMeters"]["x"] = "2.25".into();
    shapes[0]["shape"]["positionMeters"]["z"] = "5.75".into();
    shapes[1]["shape"]["outlineMeters"] =
        json!([["1.25", "0"], ["4.25", "0"], ["4.25", "2"], ["1.25", "2"]]);
    shapes[1]["shape"]["baseElevationMeters"] = "0.25".into();
    let p = &mut shapes[2]["shape"]["positionMeters"];
    p["x"] = (p["x"].as_str().unwrap().parse::<f64>().unwrap() + 1.25)
        .to_string()
        .into();
    p["z"] = (p["z"].as_str().unwrap().parse::<f64>().unwrap() - 0.25)
        .to_string()
        .into();
    assert_eq!(after["stage"], expected);
    assert_eq!(after["lighting"], before["lighting"]);
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    let zero = doc.clone();
    translate(&mut doc, &targets, ["0", "-0.000000", "0"]).unwrap();
    assert_eq!(doc, zero);
    let mut reversed = Document::decode(&serde_json::to_vec(&before).unwrap()).unwrap();
    let mut reversed_targets = targets;
    reversed_targets.reverse();
    translate(&mut reversed, &reversed_targets, ["1.25", "0", "-0.25"]).unwrap();
    assert_eq!(root(&reversed)["stage"], after["stage"]);
}
#[test]
fn direct_or_indirect_locks_reject_every_change() {
    let (doc, targets, ids) = setup();
    for locked in [
        targets[1].clone(),
        json!({"kind":"placement","targetId":ids[1]}),
    ] {
        let mut locked_doc = doc.clone();
        edit(
            &mut locked_doc,
            json!({"op":"setEditLocks","targets":[locked],"locked":true}),
        )
        .unwrap();
        let before = locked_doc.clone();
        assert!(
            translate(&mut locked_doc, &targets, ["0.1", "0", "0"])
                .unwrap_err()
                .contains("锁定")
        );
        assert_eq!(locked_doc, before);
    }
}
#[test]
fn invalid_selection_delta_and_late_geometry_validation_roll_back_the_whole_edit() {
    let (mut doc, targets, _) = setup();
    let before = doc.clone();
    for (selection, delta) in [
        (vec![], ["1", "0", "0"]),
        (vec![targets[0].clone(); 2], ["1", "0", "0"]),
        (vec![targets[0].clone(); 257], ["1", "0", "0"]),
        (
            vec![
                targets[0].clone(),
                json!({"kind":"construction","targetId":"missing"}),
            ],
            ["1", "0", "0"],
        ),
        (
            vec![json!({"kind":"space","targetId":"missing"})],
            ["1", "0", "0"],
        ),
        (targets.clone(), ["1e2", "0", "0"]),
        (targets.clone(), ["0", "NaN", "0"]),
        (targets.clone(), ["0.0000001", "0", "0"]),
        (targets.clone(), ["200001", "0", "0"]),
        (targets.clone(), ["100000", "0", "0"]),
        (targets.clone(), ["1", "0", "10000"]),
    ] {
        assert!(translate(&mut doc, &selection, delta).is_err());
        assert_eq!(doc, before);
    }
}
#[test]
fn selected_fixture_alone_does_not_move_its_support_or_other_members() {
    let (mut doc, targets, _) = setup();
    let before = root(&doc)["stage"].clone();
    translate(&mut doc, &targets[3..], ["0", "-2", "0"]).unwrap();
    let mut expected = before;
    expected["placements"][0]["positionMeters"]["y"] = "0.125123".into();
    assert_eq!(root(&doc)["stage"], expected);
}
#[test]
fn a_single_rig_can_move_more_than_256_attached_fixtures_without_truncation() {
    let (doc, targets, _) = setup();
    let mut r = root(&doc);
    let fixture = r["lighting"]["fixtures"][0].clone();
    let patch = r["lighting"]["patches"][0].clone();
    let placement = r["stage"]["placements"][0].clone();
    r["lighting"]["fixtures"] = json!([]);
    r["lighting"]["patches"] = json!([]);
    r["stage"]["placements"] = json!([]);
    r["stage"]["attachments"] = json!([]);
    for i in 0..300 {
        let id = uuid::Uuid::new_v4().to_string();
        let mut f = fixture.clone();
        f["id"] = id.clone().into();
        let mut p = patch.clone();
        p["fixtureId"] = id.clone().into();
        p["universe"] = (1 + i / 128).into();
        p["address"] = (1 + (i % 128) * 4).into();
        let mut v = placement.clone();
        v["fixtureId"] = id.clone().into();
        r["lighting"]["fixtures"].as_array_mut().unwrap().push(f);
        r["lighting"]["patches"].as_array_mut().unwrap().push(p);
        r["stage"]["placements"].as_array_mut().unwrap().push(v);
        r["stage"]["attachments"]
            .as_array_mut()
            .unwrap()
            .push(json!({"fixtureId":id,"constructionId":targets[0]["targetId"]}));
    }
    let mut doc = Document::decode(&serde_json::to_vec(&r).unwrap()).unwrap();
    translate(&mut doc, &targets[..1], ["2", "0", "0"]).unwrap();
    assert_eq!(doc.view().stage.placements.len(), 300);
    assert!(
        doc.view()
            .stage
            .placements
            .iter()
            .all(|p| p.position_meters.x == "2")
    );
}

#[test]
fn maximum_direct_selection_is_accepted_and_one_extra_is_rejected_atomically() {
    let (doc, _, _) = setup();
    let mut r = root(&doc);
    let construction = r["stage"]["constructions"][0].clone();
    r["stage"]["constructions"] = json!([]);
    r["stage"]["attachments"] = json!([]);
    let mut targets = Vec::new();
    for _ in 0..256 {
        let id = uuid::Uuid::new_v4().to_string();
        let mut c = construction.clone();
        c["id"] = id.clone().into();
        targets.push(json!({"kind":"construction","targetId":id}));
        r["stage"]["constructions"].as_array_mut().unwrap().push(c);
    }
    let mut doc = Document::decode(&serde_json::to_vec(&r).unwrap()).unwrap();
    translate(&mut doc, &targets, ["0.25", "0", "0"]).unwrap();
    assert!(
        root(&doc)["stage"]["constructions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["shape"]["positionMeters"]["x"] == "1.25")
    );
    let before = doc.clone();
    targets.push(json!({"kind":"placement","targetId":doc.view().fixtures[0].id}));
    assert!(translate(&mut doc, &targets, ["1", "0", "0"]).is_err());
    assert_eq!(doc, before);
}
