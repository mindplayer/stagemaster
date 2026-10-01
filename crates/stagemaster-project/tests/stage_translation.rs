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
fn translate(doc: &mut Document, ids: &[String], delta: [&str; 3]) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"translatePlacements","fixtureIds":ids,
        "deltaMeters":{"x":delta[0],"y":delta[1],"z":delta[2]}}),
    )
}
#[test]
fn translation_preserves_unselected_objects_relationships_and_zero_axes() {
    let (mut doc, ids) = setup();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"前架","shape":{"kind":"rig","rigKind":"truss","spaceId":null,"positionMeters":{"x":"1","y":"2","z":"6"},"yawDegrees":"0","lengthMeters":"3","widthMeters":"0.3","heightMeters":"0.3"}})).unwrap();
    let rig = doc.view().stage.constructions[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"attachFixtures","constructionId":rig,"fixtureIds":[ids[0]],"layout":null}),
    )
    .unwrap();
    let before = doc.view().stage;
    translate(&mut doc, &ids[..2], ["1.25", "0", "-0.625"]).unwrap();
    let after = doc.view().stage;
    for (index, (a, b)) in after.placements[..2]
        .iter()
        .zip(&before.placements[..2])
        .enumerate()
    {
        assert_eq!(a.position_meters.x, ["1.25", "2.25"][index]);
        assert_eq!(a.position_meters.z, ["2.375", "3.375"][index]);
        assert_eq!(a.position_meters.y, "2.125");
        assert_eq!(a.space_id, b.space_id);
        assert_eq!(
            serde_json::to_value(&a.rotation_degrees_xyz).unwrap(),
            serde_json::to_value(&b.rotation_degrees_xyz).unwrap()
        );
    }
    assert_eq!(
        serde_json::to_value(&after.placements[2]).unwrap(),
        serde_json::to_value(&before.placements[2]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&after.attachments).unwrap(),
        serde_json::to_value(&before.attachments).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&after.constructions).unwrap(),
        serde_json::to_value(&before.constructions).unwrap()
    );
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    let zero = doc.clone();
    translate(&mut doc, &ids, ["0", "-0.000000", "0"]).unwrap();
    assert_eq!(doc, zero);
}
#[test]
fn any_locked_missing_or_outside_member_rejects_the_whole_group() {
    let (mut doc, ids) = setup();
    let original = doc.clone();
    for (selection, delta) in [
        (vec![], ["1", "0", "0"]),
        (vec![ids[0].clone(); 257], ["1", "0", "0"]),
        (vec![ids[0].clone(); 2], ["1", "0", "0"]),
        (vec![ids[0].clone(), "missing".into()], ["1", "0", "0"]),
        (ids.clone(), ["100000", "0", "0"]),
        (ids.clone(), ["200001", "0", "0"]),
        (ids.clone(), ["NaN", "0", "0"]),
        (ids.clone(), ["1e2", "0", "0"]),
        (ids.clone(), ["0.0000001", "0", "0"]),
    ] {
        assert!(translate(&mut doc, &selection, delta).is_err());
        assert_eq!(doc, original);
    }
    edit(&mut doc,json!({"op":"setEditLocks","targets":[{"kind":"placement","targetId":ids[1]}],"locked":true})).unwrap();
    let locked = doc.clone();
    assert!(
        translate(&mut doc, &ids, ["1", "0", "0"])
            .unwrap_err()
            .contains("锁定")
    );
    assert_eq!(doc, locked);
}

#[test]
fn maximum_group_and_signed_world_boundaries_are_inclusive() {
    let (doc, _) = setup();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let fixture = root["lighting"]["fixtures"][0].clone();
    let patch = root["lighting"]["patches"][0].clone();
    let placement = root["stage"]["placements"][0].clone();
    root["lighting"]["fixtures"] = json!([]);
    root["lighting"]["patches"] = json!([]);
    root["stage"]["placements"] = json!([]);
    let mut ids = Vec::new();
    for index in 0..256_u32 {
        let id = uuid::Uuid::new_v4().to_string();
        let mut fixture = fixture.clone();
        fixture["id"] = id.clone().into();
        let mut patch = patch.clone();
        patch["fixtureId"] = id.clone().into();
        patch["universe"] = (1 + index / 128).into();
        patch["address"] = (1 + (index % 128) * 4).into();
        let mut placement = placement.clone();
        placement["fixtureId"] = id.clone().into();
        placement["positionMeters"]["x"] = "-100000".into();
        root["lighting"]["fixtures"]
            .as_array_mut()
            .unwrap()
            .push(fixture);
        root["lighting"]["patches"]
            .as_array_mut()
            .unwrap()
            .push(patch);
        root["stage"]["placements"]
            .as_array_mut()
            .unwrap()
            .push(placement);
        ids.push(id);
    }
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let original = doc.clone();
    translate(&mut doc, &ids, ["200000", "0", "0"]).unwrap();
    assert!(
        doc.view()
            .stage
            .placements
            .iter()
            .all(|p| p.position_meters.x == "100000")
    );
    let moved = doc.clone();
    assert!(translate(&mut doc, &ids, ["0.000001", "0", "0"]).is_err());
    assert_eq!(doc, moved);
    translate(&mut doc, &ids, ["-200000", "0", "0"]).unwrap();
    assert_eq!(doc, original);
}
