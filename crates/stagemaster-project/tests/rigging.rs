use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, c: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(c).unwrap(),
    })
}
fn shape() -> Value {
    json!({"kind":"rig","rigKind":"truss","spaceId":null,"positionMeters":{"x":"4","y":"3","z":"5"},"yawDegrees":"0","lengthMeters":"6","widthMeters":"0.3","heightMeters":"0.4"})
}
fn setup() -> (Document, String, Vec<String>) {
    let mut d = Document::new("挂灯测试").unwrap();
    let v = d.view();
    for i in 0..3 {
        d.edit(EditCommand::AddFixture {
            name: format!("灯{i}"),
            profile_id: v.profiles[1].id.clone(),
            domain_id: v.domains[0].id.clone(),
            universe: 1,
            address: 1 + i * 4,
        })
        .unwrap();
    }
    edit(
        &mut d,
        json!({"op":"putConstruction","id":null,"name":"前桁架","shape":shape()}),
    )
    .unwrap();
    let v = d.view();
    (
        d,
        v.stage.constructions[0].id.clone(),
        v.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
fn hang(id: &str, ids: &[String]) -> Value {
    json!({"op":"attachFixtures","constructionId":id,"fixtureIds":ids,"layout":{"startMarginMeters":"1","endMarginMeters":"1","dropMeters":"0.2"}})
}
fn root(d: &Document) -> Value {
    serde_json::from_slice(&d.encode().unwrap()).unwrap()
}
fn reject(d: &mut Document, c: Value, part: &str) {
    let before = d.encode().unwrap();
    let e = edit(d, c).unwrap_err();
    assert!(e.contains(part), "{e}");
    assert_eq!(before, d.encode().unwrap());
}
#[test]
fn hanging_order_transform_and_world_round_trip() {
    let (mut d, id, mut ids) = setup();
    ids.reverse();
    let lighting = root(&d)["lighting"].clone();
    edit(&mut d, hang(&id, &ids)).unwrap();
    let placements = d.view().stage.placements;
    assert_eq!(
        placements
            .iter()
            .map(|p| p.fixture_id.clone())
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        placements
            .iter()
            .map(|p| p.position_meters.x.as_str())
            .collect::<Vec<_>>(),
        ["2", "4", "6"]
    );
    assert!(
        placements
            .iter()
            .all(|p| p.position_meters.y == "3" && p.position_meters.z == "4.6")
    );
    let mut s = shape();
    s["yawDegrees"] = json!("90");
    s["positionMeters"] = json!({"x":"8","y":"7","z":"6"});
    edit(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"旋转","shape":s}),
    )
    .unwrap();
    let p = d.view().stage.placements;
    assert_eq!(
        p.iter()
            .map(|p| p.position_meters.y.as_str())
            .collect::<Vec<_>>(),
        ["5", "7", "9"]
    );
    assert!(p.iter().all(|p| p.position_meters.x == "8"
        && p.position_meters.z == "5.6"
        && p.rotation_degrees_xyz.z == "90"));
    assert_eq!(root(&d)["lighting"], lighting);
    let reopened = Document::decode(&d.encode().unwrap()).unwrap();
    assert_eq!(d, reopened);
    assert_eq!(reopened.view().stage.attachments.len(), 3);
}
#[test]
fn individual_position_changes_remain_attached_and_resize_does_not_respace() {
    let (mut d, id, ids) = setup();
    edit(&mut d, hang(&id, &ids)).unwrap();
    let mut p = d.view().stage.placements[0].clone();
    p.position_meters.x = "1.75".into();
    p.rotation_degrees_xyz.x = "20".into();
    edit(&mut d, json!({"op":"putPlacement","placement":p})).unwrap();
    let old = root(&d)["stage"]["placements"].clone();
    let mut s = shape();
    s["lengthMeters"] = json!("8");
    edit(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"加长","shape":s}),
    )
    .unwrap();
    assert_eq!(root(&d)["stage"]["placements"], old);
    s["positionMeters"]["x"] = json!("5");
    edit(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"移动","shape":s}),
    )
    .unwrap();
    assert_eq!(d.view().stage.placements[0].position_meters.x, "2.75");
    assert_eq!(d.view().stage.placements[0].rotation_degrees_xyz.x, "20");
}
#[test]
fn deletion_detach_and_copy_preserve_fixture_identity() {
    let (mut d, id, ids) = setup();
    edit(&mut d, hang(&id, &ids)).unwrap();
    reject(&mut d, json!({"op":"removeConstruction","id":id}), "仍挂接");
    edit(
        &mut d,
        json!({"op":"duplicateConstruction","id":id,"name":"副本"}),
    )
    .unwrap();
    assert_eq!(d.view().stage.attachments.len(), 3);
    assert_eq!(d.view().fixtures.len(), 3);
    let placements = root(&d)["stage"]["placements"].clone();
    edit(
        &mut d,
        json!({"op":"removeConstruction","id":id,"detachFixtures":true}),
    )
    .unwrap();
    assert!(d.view().stage.attachments.is_empty());
    assert_eq!(root(&d)["stage"]["placements"], placements);
    let copy = d.view().stage.constructions[0].id.clone();
    edit(
        &mut d,
        json!({"op":"attachFixtures","constructionId":copy,"fixtureIds":ids,"layout":null}),
    )
    .unwrap();
    assert_eq!(root(&d)["stage"]["placements"], placements);
    edit(&mut d, json!({"op":"removePlacement","fixtureId":ids[0]})).unwrap();
    assert_eq!(d.view().stage.attachments.len(), 2);
    edit(
        &mut d,
        json!({"op":"attachFixtures","constructionId":null,"fixtureIds":[ids[1]],"layout":null}),
    )
    .unwrap();
    assert_eq!(d.view().stage.attachments.len(), 1);
}
#[test]
fn invalid_hanging_inputs_and_transform_are_atomic() {
    let (mut d, id, ids) = setup();
    reject(
        &mut d,
        json!({"op":"attachFixtures","constructionId":id,"fixtureIds":ids,"layout":null}),
        "已有灯位",
    );
    reject(
        &mut d,
        hang(&id, &[ids[0].clone(), ids[0].clone()]),
        "不重复",
    );
    reject(&mut d, hang(&id, &[]), "1–256");
    reject(&mut d, hang(&id, &vec![ids[0].clone(); 257]), "1–256");
    let mut c = hang(&id, &ids);
    c["layout"]["endMarginMeters"] = json!("5.1");
    reject(&mut d, c, "余量过大");
    edit(&mut d, hang(&id, &ids)).unwrap();
    let mut s = shape();
    s["positionMeters"]["x"] = json!("100000");
    reject(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"越界","shape":s}),
        "超出",
    );
    let mut p = d.view().stage.placements[0].clone();
    p.position_meters.x = "99999".into();
    edit(&mut d, json!({"op":"putPlacement","placement":p})).unwrap();
    let mut s = shape();
    s["positionMeters"]["x"] = json!("6");
    reject(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"灯越界","shape":s}),
        "空间数值",
    );
    let before = d.encode().unwrap();
    let commands = vec![
        json!({"op":"stage","command":{"op":"attachFixtures","constructionId":null,"fixtureIds":ids,"layout":null}}),
        json!({"op":"stage","command":{"op":"removeConstruction","id":"missing"}}),
    ];
    let command = serde_json::from_value(json!({"op":"batch","commands":commands})).unwrap();
    assert!(d.edit(command).is_err());
    assert_eq!(before, d.encode().unwrap());
}
#[test]
fn malformed_persistence_requires_capability_valid_unique_associations() {
    let (mut d, id, ids) = setup();
    edit(&mut d, hang(&id, &ids)).unwrap();
    let good = root(&d);
    let mut bad = good.clone();
    bad["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["key"] != "stage.rigging");
    assert!(
        Document::decode(&serde_json::to_vec(&bad).unwrap())
            .unwrap_err()
            .contains("挂接能力")
    );
    for change in 0..3 {
        let mut bad = good.clone();
        match change {
            0 => {
                bad["stage"]["attachments"][1] = bad["stage"]["attachments"][0].clone();
            }
            1 => {
                bad["stage"]["attachments"][0]["constructionId"] = json!(ids[0]);
            }
            _ => {
                bad["stage"]["placements"] = json!([]);
            }
        }
        assert!(Document::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let d = Document::new("旧文件").unwrap();
    assert!(
        Document::decode(&d.encode().unwrap())
            .unwrap()
            .view()
            .stage
            .attachments
            .is_empty()
    );
}
#[test]
fn space_membership_follows_rig_but_room_geometry_does_not_move_members() {
    let (mut d, id, ids) = setup();
    edit(&mut d, hang(&id, &ids)).unwrap();
    edit(&mut d,json!({"op":"putSpace","id":null,"name":"房间","outlineMeters":[["0","0"],["8","0"],["8","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"8"})).unwrap();
    let space = d.view().stage.spaces[0].id.clone();
    let mut s = shape();
    s["spaceId"] = json!(space);
    edit(
        &mut d,
        json!({"op":"putConstruction","id":id,"name":"归属房间","shape":s}),
    )
    .unwrap();
    assert!(
        d.view()
            .stage
            .placements
            .iter()
            .all(|p| p.space_id.as_ref() == Some(&space))
    );
    let mut p = d.view().stage.placements[0].clone();
    p.space_id = None;
    reject(
        &mut d,
        json!({"op":"putPlacement","placement":p}),
        "同一空间",
    );
    reject(
        &mut d,
        json!({"op":"removeSpace","id":space,"detachMembers":false}),
        "解除归属",
    );
    edit(
        &mut d,
        json!({"op":"removeSpace","id":space,"detachMembers":true}),
    )
    .unwrap();
    assert!(
        d.view()
            .stage
            .placements
            .iter()
            .all(|p| p.space_id.is_none())
    );
    assert_eq!(d.view().stage.attachments.len(), 3);
}
