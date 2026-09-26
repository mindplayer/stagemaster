use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn room(id: Option<&str>, name: &str) -> Value {
    json!({"op":"putSpace","id":id,"name":name,"outlineMeters":[["0","0"],["8","0"],["8","3"],["3","3"],["3","7"],["0","7"]],"floorElevationMeters":"1.2","clearHeightMeters":"4.5"})
}
fn setup() -> (Document, String, String) {
    let mut doc = Document::new("多房间验证").unwrap();
    let view = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "顶灯".into(),
        profile_id: view.profiles[1].id.clone(),
        domain_id: view.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    let fixture = doc.view().fixtures[0].id.clone();
    edit(&mut doc, room(None, "演出厅")).unwrap();
    let space = doc.view().stage.spaces[0].id.clone();
    edit(&mut doc,json!({"op":"putPlacement","placement":{"fixtureId":fixture,"spaceId":space,"positionMeters":{"x":"2.1","y":"3.2","z":"5.5"},"rotationDegreesXYZ":{"x":"12","y":"-30","z":"90"}}})).unwrap();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"厅围护","shape":{"kind":"enclosure","spaceId":space,"wallThicknessMeters":"0.2","floorThicknessMeters":"0.1","ceilingThicknessMeters":"0.1"}})).unwrap();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"台","shape":{"kind":"platform","spaceId":space,"outlineMeters":[["0","0"],["2","0"],["2","1"],["0","1"]],"baseElevationMeters":"1.2","heightMeters":"0.6"}})).unwrap();
    (doc, space, fixture)
}
fn reject(doc: &mut Document, command: Value, message: &str) {
    let before = doc.encode().unwrap();
    let error = edit(doc, command).unwrap_err();
    assert!(error.contains(message), "{error}");
    assert_eq!(doc.encode().unwrap(), before);
}
#[test]
fn blank_lighting_stays_unchanged_until_space_edit_and_round_trip_retains_geometry() {
    let doc = Document::new("空白").unwrap();
    assert!(doc.view().stage.spaces.is_empty());
    assert!(
        serde_json::from_slice::<Value>(&doc.encode().unwrap())
            .unwrap()
            .get("stage")
            .is_none()
    );
    let (doc, _, _) = setup();
    let next = doc.next_revision();
    let reopened = Document::decode(&next.encode().unwrap()).unwrap();
    assert_eq!(next, reopened);
    assert_eq!(reopened.view().stage.spaces[0].outline_meters.len(), 6);
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for key in ["stage.layout", "stage.spaces"] {
        assert!(
            root["requires"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["key"] == key)
        );
    }
}
#[test]
fn room_and_membership_changes_never_implicitly_move_fixtures_or_platforms() {
    let (mut doc, space, _) = setup();
    let placements = serde_json::to_value(doc.view().stage.placements).unwrap();
    let constructions = serde_json::to_value(doc.view().stage.constructions).unwrap();
    let mut update = room(Some(&space), "改名");
    update["floorElevationMeters"] = json!("-2");
    update["clearHeightMeters"] = json!("6");
    edit(&mut doc, update).unwrap();
    assert_eq!(
        serde_json::to_value(doc.view().stage.placements).unwrap(),
        placements
    );
    assert_eq!(
        serde_json::to_value(doc.view().stage.constructions).unwrap(),
        constructions
    );
}
#[test]
fn removing_room_requires_explicit_detach_and_keeps_world_members() {
    let (mut doc, space, fixture) = setup();
    reject(
        &mut doc,
        json!({"op":"removeSpace","id":space,"detachMembers":false}),
        "解除归属",
    );
    let before = serde_json::to_value(doc.view().stage.placements[0].clone()).unwrap();
    edit(
        &mut doc,
        json!({"op":"removeSpace","id":space,"detachMembers":true}),
    )
    .unwrap();
    let view = doc.view();
    assert!(view.stage.spaces.is_empty());
    assert_eq!(view.stage.constructions.len(), 1);
    assert_eq!(view.fixtures[0].id, fixture);
    assert_eq!(view.fixtures[0].address, Some(1));
    let after = serde_json::to_value(&view.stage.placements[0]).unwrap();
    assert!(after["spaceId"].is_null());
    assert_eq!(before["positionMeters"], after["positionMeters"]);
    assert_eq!(before["rotationDegreesXYZ"], after["rotationDegreesXYZ"]);
    assert!(
        serde_json::to_value(&view.stage.constructions[0]).unwrap()["shape"]["spaceId"].is_null()
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn duplicate_room_copies_enclosure_without_duplicating_fixture_or_platform() {
    let (mut doc, space, _) = setup();
    edit(
        &mut doc,
        json!({"op":"duplicateSpace","id":space,"name":"副厅"}),
    )
    .unwrap();
    let stage = doc.view().stage;
    assert_eq!(stage.spaces.len(), 2);
    assert_ne!(stage.spaces[0].id, stage.spaces[1].id);
    assert_eq!(stage.constructions.len(), 3);
    assert_eq!(stage.placements.len(), 1);
    assert_eq!(
        serde_json::to_value(&stage.constructions[2]).unwrap()["shape"]["spaceId"],
        stage.spaces[1].id
    );
    let platform = stage.constructions[1].id.clone();
    edit(
        &mut doc,
        json!({"op":"duplicateConstruction","id":platform,"name":"台二"}),
    )
    .unwrap();
    assert_ne!(
        doc.view().stage.constructions[1].id,
        doc.view().stage.constructions[3].id
    );
    reject(
        &mut doc,
        json!({"op":"duplicateConstruction","id":stage.constructions[0].id,"name":"重复围护"}),
        "随空间复制",
    );
}
#[test]
fn geometry_and_reference_failures_are_atomic_and_bounded() {
    let (mut doc, space, _) = setup();
    for points in [
        json!([["0", "0"], ["4", "4"], ["0", "4"], ["4", "0"]]),
        json!([["0", "0"], ["100001", "0"], ["0", "4"]]),
    ] {
        let mut command = room(Some(&space), "错误");
        command["outlineMeters"] = points;
        reject(&mut doc, command, "空间");
    }
    for value in ["-1", "0", "1001"] {
        let mut command = room(Some(&space), "错误");
        command["clearHeightMeters"] = json!(value);
        reject(&mut doc, command, "空间数值");
    }
    let mut command = room(Some(&space), "错误");
    command["clearHeightMeters"] = Value::Null;
    reject(&mut doc, command, "净高");
    reject(&mut doc, room(Some("missing"), "丢失"), "不存在");
    let mut placement = serde_json::to_value(doc.view().stage.placements[0].clone()).unwrap();
    placement["fixtureId"] = json!("00000000-0000-4000-8000-000000000001");
    reject(
        &mut doc,
        json!({"op":"putPlacement","placement":placement}),
        "灯具不存在",
    );
    placement["fixtureId"] = json!(doc.view().fixtures[0].id);
    placement["spaceId"] = json!("00000000-0000-4000-8000-000000000001");
    reject(
        &mut doc,
        json!({"op":"putPlacement","placement":placement}),
        "所属空间不存在",
    );
    placement["spaceId"] = Value::Null;
    for value in ["3601", "NaN", "1e300", "-0", "1.00"] {
        placement["rotationDegreesXYZ"]["x"] = json!(value);
        let before = doc.encode().unwrap();
        assert!(edit(&mut doc, json!({"op":"putPlacement","placement":placement})).is_err());
        assert_eq!(doc.encode().unwrap(), before);
    }
}
#[test]
fn fixtures_cannot_be_removed_leaving_dangling_placements() {
    let (mut doc, _, fixture) = setup();
    let before = doc.encode().unwrap();
    assert!(
        doc.edit(EditCommand::RemoveFixture {
            id: fixture.clone()
        })
        .unwrap_err()
        .contains("移除灯位")
    );
    assert_eq!(doc.encode().unwrap(), before);
    edit(
        &mut doc,
        json!({"op":"removePlacement","fixtureId":fixture}),
    )
    .unwrap();
    doc.edit(EditCommand::RemoveFixture { id: fixture })
        .unwrap();
    assert!(doc.view().fixtures.is_empty());
}
#[test]
fn malformed_loaded_stage_rejects_capabilities_duplicates_and_invalid_geometry() {
    let (doc, _, _) = setup();
    let source: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for case in 0..5 {
        let mut root = source.clone();
        match case {
            0 => root["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c["key"] != "stage.spaces"),
            1 => {
                let p = root["stage"]["placements"][0].clone();
                root["stage"]["placements"].as_array_mut().unwrap().push(p);
            }
            2 => root["stage"]["spaces"][0]["id"] = root["lighting"]["fixtures"][0]["id"].clone(),
            3 => {
                root["stage"]["spaces"][0]["outlineMeters"] =
                    json!([["0", "0"], ["4", "4"], ["0", "4"], ["4", "0"]]);
            }
            _ => root["stage"]["unknown"] = json!(true),
        }
        assert!(Document::decode(&serde_json::to_vec(&root).unwrap()).is_err());
    }
}
#[test]
fn atomic_batch_does_not_keep_first_move_when_second_placement_fails() {
    let (mut doc, _, _) = setup();
    let before = doc.encode().unwrap();
    let mut moved = serde_json::to_value(doc.view().stage.placements[0].clone()).unwrap();
    moved["positionMeters"]["x"] = json!("8");
    let mut invalid = moved.clone();
    invalid["fixtureId"] = json!("00000000-0000-4000-8000-000000000001");
    assert!(doc.edit(serde_json::from_value(json!({"op":"batch","commands":[{"op":"stage","command":{"op":"putPlacement","placement":moved}},{"op":"stage","command":{"op":"putPlacement","placement":invalid}}]})).unwrap()).is_err());
    assert_eq!(doc.encode().unwrap(), before);
}
