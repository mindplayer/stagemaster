use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, c: Value) -> Result<(), String> {
    doc.edit(EditCommand::Stage {
        command: serde_json::from_value(c).unwrap(),
    })
}
fn lock(kind: &str, id: &str, locked: bool) -> Value {
    json!({"op":"setEditLocks","targets":[{"kind":kind,"targetId":id}],"locked":locked})
}
fn setup() -> (Document, String, String, String) {
    let mut doc = Document::new("场地保护").unwrap();
    let v = doc.view();
    doc.edit(EditCommand::AddFixture {
        name: "面光 1".into(),
        profile_id: v.profiles[0].id.clone(),
        domain_id: v.domains[0].id.clone(),
        universe: 1,
        address: 1,
    })
    .unwrap();
    edit(&mut doc,json!({"op":"putSpace","id":null,"name":"大厅","outlineMeters":[["0","0"],["10","0"],["10","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"7"})).unwrap();
    let space = doc.view().stage.spaces[0].id.clone();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"前桁架","shape":{"kind":"rig","rigKind":"truss","spaceId":space,"positionMeters":{"x":"5","y":"2","z":"6"},"yawDegrees":"0","lengthMeters":"8","widthMeters":"0.3","heightMeters":"0.3"}})).unwrap();
    let rig = doc.view().stage.constructions[0].id.clone();
    let fixture = doc.view().fixtures[0].id.clone();
    edit(&mut doc,json!({"op":"attachFixtures","constructionId":rig,"fixtureIds":[fixture],"layout":{"startMarginMeters":"1","endMarginMeters":"1","dropMeters":"0.2"}})).unwrap();
    (doc, space, rig, fixture)
}
fn rejected(doc: &mut Document, command: Value) {
    let before = doc.encode().unwrap();
    assert!(edit(doc, command).unwrap_err().contains("锁定"));
    assert_eq!(doc.encode().unwrap(), before);
}
#[test]
fn fixture_lock_blocks_direct_and_indirect_edits_atomically_but_not_lighting() {
    let (mut doc, space, rig, fixture) = setup();
    edit(&mut doc, lock("placement", &fixture, true)).unwrap();
    let mut placement = doc.view().stage.placements[0].clone();
    placement.position_meters.x = "8".into();
    rejected(&mut doc, json!({"op":"putPlacement","placement":placement}));
    rejected(
        &mut doc,
        json!({"op":"removePlacement","fixtureId":fixture}),
    );
    rejected(
        &mut doc,
        json!({"op":"attachFixtures","constructionId":null,"fixtureIds":[fixture],"layout":null}),
    );
    rejected(
        &mut doc,
        json!({"op":"removeConstruction","id":rig,"detachFixtures":true}),
    );
    rejected(
        &mut doc,
        json!({"op":"removeSpace","id":space,"detachMembers":true}),
    );
    let mut shape = serde_json::to_value(doc.view().stage.constructions[0].shape.clone()).unwrap();
    shape["positionMeters"]["x"] = "6".into();
    rejected(
        &mut doc,
        json!({"op":"putConstruction","id":rig,"name":"前桁架","shape":shape}),
    );
    doc.edit(EditCommand::AddScene {
        name: "灯光仍可编辑".into(),
    })
    .unwrap();
    let scene = doc.view().scenes[0].id.clone();
    doc.edit(serde_json::from_value(json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":"dimmer","mode":"literal","value":32768})).unwrap()).unwrap();
    assert!(doc.compile_scene(&scene).is_ok());
    let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
    assert_eq!(reopened, doc);
}
#[test]
fn explicit_unlock_edit_relock_batch_is_allowed_and_reverse_order_is_atomic() {
    let (mut doc, _, _, fixture) = setup();
    edit(&mut doc, lock("placement", &fixture, true)).unwrap();
    let mut p = doc.view().stage.placements[0].clone();
    p.position_meters.x = "6".into();
    let move_command = json!({"op":"stage","command":{"op":"putPlacement","placement":p}});
    let unlock = json!({"op":"stage","command":lock("placement",&fixture,false)});
    let relock = json!({"op":"stage","command":lock("placement",&fixture,true)});
    let before = doc.clone();
    assert!(
        doc.edit(
            serde_json::from_value(json!({"op":"batch","commands":[move_command,unlock]})).unwrap()
        )
        .is_err()
    );
    assert_eq!(doc, before);
    doc.edit(
        serde_json::from_value(json!({"op":"batch","commands":[unlock,move_command,relock]}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(doc.view().stage.placements[0].position_meters.x, "6");
    assert_eq!(doc.view().stage.edit_locks.len(), 1);
    edit(&mut doc, lock("placement", &fixture, false)).unwrap();
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert!(root["stage"].get("editLocks").is_none());
    assert!(
        !root["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["key"] == "stage.edit-locks")
    );
}
#[test]
fn room_lock_allows_member_edits_and_locked_copy_is_independent() {
    let (mut doc, space, rig, _) = setup();
    edit(&mut doc, lock("space", &space, true)).unwrap();
    let mut p = doc.view().stage.placements[0].clone();
    p.position_meters.x = "6".into();
    edit(&mut doc, json!({"op":"putPlacement","placement":p})).unwrap();
    edit(&mut doc, lock("construction", &rig, true)).unwrap();
    edit(
        &mut doc,
        json!({"op":"duplicateConstruction","id":rig,"name":"副本"}),
    )
    .unwrap();
    let copy = doc.view().stage.constructions[1].id.clone();
    assert!(
        !doc.view()
            .stage
            .edit_locks
            .iter()
            .any(|v| v.target_id == copy)
    );
    edit(
        &mut doc,
        json!({"op":"removeConstruction","id":copy,"detachFixtures":false}),
    )
    .unwrap();
    edit(
        &mut doc,
        json!({"op":"duplicateSpace","id":space,"name":"房间副本"}),
    )
    .unwrap();
    assert_eq!(doc.view().stage.edit_locks.len(), 2);
}
#[test]
fn enclosure_protection_includes_derived_geometry_and_invalid_locks_are_rejected() {
    let (mut doc, space, _, _) = setup();
    edit(&mut doc,json!({"op":"putConstruction","id":null,"name":"围护","shape":{"kind":"enclosure","spaceId":space,"wallThicknessMeters":"0.2","floorThicknessMeters":"0.1","ceilingThicknessMeters":null}})).unwrap();
    let enclosure = doc.view().stage.constructions[1].id.clone();
    edit(&mut doc, lock("construction", &enclosure, true)).unwrap();
    let mut s = serde_json::to_value(doc.view().stage.spaces[0].clone()).unwrap();
    s["op"] = "putSpace".into();
    s["clearHeightMeters"] = "8".into();
    rejected(&mut doc, s);
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for variant in 0..4 {
        let mut bad = root.clone();
        match variant {
            0 => bad["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v["key"] != "stage.edit-locks"),
            1 => {
                let duplicate = bad["stage"]["editLocks"][0].clone();
                bad["stage"]["editLocks"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
            2 => {
                bad["stage"]["editLocks"][0]["targetId"] =
                    "a0000000-0000-4000-8000-000000000001".into();
            }
            _ => bad["stage"]["editLocks"] = json!([]),
        }
        assert!(Document::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let old = Document::new("旧工程").unwrap();
    assert_eq!(old, Document::decode(&old.encode().unwrap()).unwrap());
    let mut old = old;
    assert!(
        edit(
            &mut old,
            json!({"op":"setEditLocks","targets":[],"locked":true})
        )
        .is_err()
    );
}

#[test]
fn malformed_lock_commands_are_atomic_and_exact_noop_geometry_remains_allowed() {
    let (mut doc, _, _, fixture) = setup();
    edit(&mut doc, lock("placement", &fixture, true)).unwrap();
    let placement = doc.view().stage.placements[0].clone();
    edit(&mut doc, json!({"op":"putPlacement","placement":placement})).unwrap();
    let original = doc.clone();
    let target = json!({"kind":"placement","targetId":fixture});
    for targets in [
        json!([target, target]),
        json!([{"kind":"space","targetId":fixture}]),
        json!(vec![target; 1601]),
    ] {
        assert!(
            edit(
                &mut doc,
                json!({"op":"setEditLocks","targets":targets,"locked":false})
            )
            .is_err()
        );
        assert_eq!(doc, original);
    }
}
