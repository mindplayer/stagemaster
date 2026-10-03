use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};

fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn setup() -> (Document, Vec<String>) {
    let mut doc = Document::new("整组灯位").unwrap();
    let view = doc.view();
    for i in 0..3 {
        doc.edit(EditCommand::AddFixture {
            name: format!("灯 {i}"),
            profile_id: view.profiles[0].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address: 1 + i * 4,
        })
        .unwrap();
    }
    let ids = doc
        .view()
        .fixtures
        .iter()
        .map(|f| f.id.clone())
        .collect::<Vec<_>>();
    for (i, id) in ids.iter().enumerate() {
        edit(
            &mut doc,
            json!({"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,
            "positionMeters":{"x":i.to_string(),"y":"2.125","z":(3+i).to_string()},
            "rotationDegreesXYZ":{"x":"180","y":"7","z":"35"}}}),
        )
        .unwrap();
    }
    (doc, ids)
}
fn transform(doc: &mut Document, ids: &[String], yaw: &str, scale: &str) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"transformPlacements", "fixtureIds":ids,"yawDegrees":yaw,"spacingScale":scale}),
    )
}
#[test]
fn group_rotation_and_spacing_use_shared_center_preserve_relationships_and_round_trip() {
    let (mut doc, ids) = setup();
    let before = doc.view().stage;
    transform(&mut doc, &ids, "90", "2").unwrap();
    let after = doc.view().stage;
    for (i, p) in after.placements.iter().enumerate() {
        assert_eq!(p.position_meters.x, "1");
        assert_eq!(p.position_meters.y, ["0.125", "2.125", "4.125"][i]);
        assert_eq!(p.position_meters.z, ["2", "4", "6"][i]);
        assert_eq!(p.rotation_degrees_xyz.z, "125");
        assert_eq!(
            p.rotation_degrees_xyz.x,
            before.placements[i].rotation_degrees_xyz.x
        );
        assert_eq!(
            p.rotation_degrees_xyz.y,
            before.placements[i].rotation_degrees_xyz.y
        );
        assert_eq!(p.space_id, before.placements[i].space_id);
    }
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    transform(&mut doc, &ids, "-90", "0.5").unwrap();
    assert_eq!(
        serde_json::to_value(doc.view().stage).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}
#[test]
fn single_spacing_full_turn_and_identity_leave_document_unchanged() {
    let (mut doc, ids) = setup();
    let before = doc.clone();
    for yaw in ["0", "360", "-360"] {
        transform(&mut doc, &ids, yaw, "1").unwrap();
        assert_eq!(doc, before);
    }
    transform(&mut doc, &ids[..1], "0", "100").unwrap();
    assert_eq!(doc, before);
    transform(&mut doc, &ids[..1], "180", "1").unwrap();
    assert_eq!(
        doc.view().stage.placements[0].rotation_degrees_xyz.z,
        "-145"
    );
    assert_eq!(doc.view().stage.placements[0].position_meters.x, "0");
    assert_eq!(
        serde_json::to_value(&doc.view().stage.placements[1..]).unwrap(),
        serde_json::to_value(&before.view().stage.placements[1..]).unwrap()
    );
}
#[test]
fn invalid_or_locked_member_rejects_whole_group() {
    let (mut doc, ids) = setup();
    let before = doc.clone();
    for (yaw, scale) in [
        ("361", "1"),
        ("NaN", "1"),
        ("1e2", "1"),
        ("0.0000001", "1"),
        ("0", "0"),
        ("0", "-1"),
        ("0", "100.01"),
        ("0", "Infinity"),
    ] {
        assert!(transform(&mut doc, &ids, yaw, scale).is_err());
        assert_eq!(doc, before);
    }
    for ids in [
        vec![],
        vec![ids[0].clone(); 2],
        vec![ids[0].clone(); 257],
        vec![ids[0].clone(), "missing".into()],
    ] {
        assert!(transform(&mut doc, &ids, "90", "1").is_err());
        assert_eq!(doc, before);
    }
    edit(&mut doc,json!({"op":"setEditLocks","targets":[{"kind":"placement","targetId":ids[1]}],"locked":true})).unwrap();
    let locked = doc.clone();
    assert!(
        transform(&mut doc, &ids, "90", "2")
            .unwrap_err()
            .contains("锁定")
    );
    assert_eq!(doc, locked);
}
#[test]
fn source_order_does_not_change_pivot_and_bounds_reject_atomically() {
    let (mut doc, mut ids) = setup();
    let mut reversed = doc.clone();
    transform(&mut doc, &ids, "33.125", "1.75").unwrap();
    ids.reverse();
    transform(&mut reversed, &ids, "33.125", "1.75").unwrap();
    assert_eq!(
        doc.view()
            .stage
            .placements
            .iter()
            .map(|p| serde_json::to_value(p).unwrap())
            .collect::<Vec<_>>(),
        reversed
            .view()
            .stage
            .placements
            .iter()
            .map(|p| serde_json::to_value(p).unwrap())
            .collect::<Vec<_>>()
    );
    let p = doc.view().stage.placements[0].clone();
    let mut value = serde_json::to_value(p).unwrap();
    value["positionMeters"]["x"] = "99999".into();
    edit(&mut doc, json!({"op":"putPlacement","placement":value})).unwrap();
    let before = doc.clone();
    assert!(transform(&mut doc, &ids, "0", "3").is_err());
    assert_eq!(doc, before);
}

#[test]
fn maximum_group_keeps_all_members_and_attachment_identity() {
    let (mut doc, ids) = setup();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"灯杆","shape":{"kind":"rig","rigKind":"truss","spaceId":null,"positionMeters":{"x":"1","y":"2","z":"6"},"yawDegrees":"0","lengthMeters":"3","widthMeters":"0.3","heightMeters":"0.3"}})).unwrap();
    let rig = doc.view().stage.constructions[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"attachFixtures","constructionId":rig,"fixtureIds":[ids[0]],"layout":null}),
    )
    .unwrap();
    let before = doc.view().stage;
    transform(&mut doc, &ids, "45", "2").unwrap();
    assert_eq!(
        serde_json::to_value(doc.view().stage.attachments).unwrap(),
        serde_json::to_value(before.attachments).unwrap()
    );
    assert_eq!(
        serde_json::to_value(doc.view().stage.constructions).unwrap(),
        serde_json::to_value(before.constructions).unwrap()
    );
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let fixture = root["lighting"]["fixtures"][0].clone();
    let patch = root["lighting"]["patches"][0].clone();
    let placement = root["stage"]["placements"][0].clone();
    root["lighting"]["fixtures"] = json!([]);
    root["lighting"]["patches"] = json!([]);
    root["stage"]["placements"] = json!([]);
    root["stage"]["attachments"] = json!([]);
    let mut ids = Vec::new();
    for index in 0..256_u32 {
        let id = uuid::Uuid::new_v4().to_string();
        let mut f = fixture.clone();
        f["id"] = id.clone().into();
        let mut p = patch.clone();
        p["fixtureId"] = id.clone().into();
        p["universe"] = (1 + index / 128).into();
        p["address"] = (1 + (index % 128) * 4).into();
        let mut v = placement.clone();
        v["fixtureId"] = id.clone().into();
        v["positionMeters"]["x"] = index.to_string().into();
        root["lighting"]["fixtures"].as_array_mut().unwrap().push(f);
        root["lighting"]["patches"].as_array_mut().unwrap().push(p);
        root["stage"]["placements"].as_array_mut().unwrap().push(v);
        ids.push(id);
    }
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    transform(&mut doc, &ids, "90", "2").unwrap();
    assert_eq!(doc.view().stage.placements.len(), 256);
    assert!(
        doc.view()
            .stage
            .placements
            .iter()
            .all(|p| p.position_meters.x == "127.5")
    );
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}
