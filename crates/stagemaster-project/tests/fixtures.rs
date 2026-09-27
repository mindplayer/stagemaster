use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand};
fn apply(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Fixture {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn definition() -> Value {
    json!({"name":"展灯 16 位","manufacturer":"测试厂","model":"P16","mode":"六通道","footprint":6,"channels":[
        {"attribute":"dimmer","coarse":5,"fine":1,"defaultValue":0},
        {"attribute":"red","coarse":2,"fine":null,"defaultValue":65535},
        {"attribute":"green","coarse":3,"fine":null,"defaultValue":0},
        {"attribute":"blue","coarse":4,"fine":null,"defaultValue":0}]})
}
fn add(doc: &mut Document, profile: &str, address: u16) {
    doc.edit(EditCommand::AddFixture {
        name: format!("灯 {address}"),
        profile_id: profile.into(),
        domain_id: doc.view().domains[0].id.clone(),
        universe: 1,
        address,
    })
    .unwrap();
}
fn root(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
#[test]
fn custom_nonadjacent_fine_before_coarse_round_trips_and_encodes_real_output() {
    let mut doc = Document::new("自定义灯库").unwrap();
    apply(
        &mut doc,
        json!({"op":"saveProfile","id":null,"definition":definition()}),
    )
    .unwrap();
    let p = doc.view().profiles.pop().unwrap();
    assert!(p.authorable);
    assert_eq!(p.channels[0].fine, Some(1));
    add(&mut doc, &p.id, 7);
    doc.edit(EditCommand::AddScene {
        name: "场景".into(),
    })
    .unwrap();
    let v = doc.view();
    doc.edit(serde_json::from_value(json!({"op":"setSceneValue","sceneId":v.scenes[0].id,"fixtureId":v.fixtures[0].id,"attribute":"dimmer","mode":"literal","value":0x1234})).unwrap()).unwrap();
    let compiled = doc.compile_scene(&v.scenes[0].id).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let output = compiled.output.render(player.values()).unwrap();
    assert_eq!(&output.slots[6..12], &[0x34, 255, 0, 0, 0x12, 0]);
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn profile_validation_rejects_collisions_ranges_incomplete_color_and_unknown_fields_atomically() {
    let mut doc = Document::new("边界").unwrap();
    let before = doc.clone();
    for (path, value) in [
        ("/footprint", json!(0)),
        ("/channels/0/coarse", json!(7)),
        ("/channels/0/fine", json!(5)),
        ("/channels/1/attribute", json!("pan")),
        ("/manufacturer", json!(" ")),
    ] {
        let mut d = definition();
        *d.pointer_mut(path).unwrap() = value;
        assert!(apply(&mut doc, json!({"op":"saveProfile","definition":d})).is_err());
        assert_eq!(doc, before);
    }
    let mut d = definition();
    d["unknown"] = true.into();
    assert!(
        serde_json::from_value::<EditCommand>(
            json!({"op":"fixture","command":{"op":"saveProfile","definition":d}})
        )
        .is_err()
    );
}
#[test]
fn used_profiles_are_pinned_and_unused_edits_get_new_revisions() {
    let mut doc = Document::new("修订").unwrap();
    apply(
        &mut doc,
        json!({"op":"saveProfile","definition":definition()}),
    )
    .unwrap();
    let p = doc.view().profiles.pop().unwrap();
    apply(
        &mut doc,
        json!({"op":"saveProfile","id":p.id,"definition":definition()}),
    )
    .unwrap();
    assert_ne!(p.revision, doc.view().profiles.last().unwrap().revision);
    add(&mut doc, &p.id, 1);
    let before = doc.clone();
    for cmd in [
        json!({"op":"removeProfile","id":p.id}),
        json!({"op":"saveProfile","id":p.id,"definition":definition()}),
    ] {
        assert!(apply(&mut doc, cmd).unwrap_err().contains("使用"));
        assert_eq!(before, doc);
    }
}
#[test]
fn exchange_preserves_programming_identity_and_scene_output_but_rejects_loss_or_overlap() {
    let mut doc = Document::new("换灯").unwrap();
    let original = doc.view().profiles[1].id.clone();
    add(&mut doc, &original, 1);
    add(&mut doc, &original, 5);
    doc.edit(EditCommand::AddScene {
        name: "节目".into(),
    })
    .unwrap();
    apply(
        &mut doc,
        json!({"op":"saveProfile","definition":definition()}),
    )
    .unwrap();
    let v = doc.view();
    let ids = v.fixtures.iter().map(|f| f.id.clone()).collect::<Vec<_>>();
    let before = root(&doc);
    assert!(apply(&mut doc,json!({"op":"exchange","fixtureIds":ids,"profileId":v.profiles.last().unwrap().id,"layout":null})).unwrap_err().contains("重叠"));
    assert_eq!(before, root(&doc));
    assert!(
        apply(
            &mut doc,
            json!({"op":"exchange","fixtureIds":ids,"profileId":v.profiles[0].id,"layout":null})
        )
        .unwrap_err()
        .contains("属性不一致")
    );
    apply(&mut doc,json!({"op":"exchange","fixtureIds":ids,"profileId":v.profiles.last().unwrap().id,"layout":{"universe":1,"address":501,"gap":0}})).unwrap();
    let after = root(&doc);
    for key in ["scenes", "groups", "presets", "sequences"] {
        assert_eq!(before["lighting"][key], after["lighting"][key]);
    }
    for (a, b) in before["lighting"]["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["lighting"]["fixtures"].as_array().unwrap())
    {
        for key in ["id", "name", "domainId"] {
            assert_eq!(a[key], b[key]);
        }
    }
    assert_eq!(doc.view().fixtures[1].address, Some(507));
}
#[test]
fn repatch_can_swap_occupied_selection_with_one_atomic_commit_and_exact_bounds() {
    let mut doc = Document::new("改址").unwrap();
    let p = doc.view().profiles[1].id.clone();
    add(&mut doc, &p, 1);
    add(&mut doc, &p, 5);
    let ids = doc
        .view()
        .fixtures
        .into_iter()
        .rev()
        .map(|f| f.id)
        .collect::<Vec<_>>();
    apply(
        &mut doc,
        json!({"op":"repatch","fixtureIds":ids,"layout":{"universe":1,"address":1,"gap":0}}),
    )
    .unwrap();
    assert_eq!(doc.view().fixtures[0].address, Some(5));
    let before = doc.clone();
    for (selected, addr, gap) in [
        (ids.clone(), 506, 0),
        (ids.clone(), 1, 512),
        (vec![ids[0].clone(), ids[0].clone()], 1, 0),
        (vec![], 1, 0),
        (vec!["missing".into()], 1, 0),
    ] {
        assert!(apply(&mut doc,json!({"op":"repatch","fixtureIds":selected,"layout":{"universe":1,"address":addr,"gap":gap}})).is_err());
        assert_eq!(doc, before);
    }
    let error = apply(
        &mut doc,
        json!({"op":"repatch","fixtureIds":[ids[0]],"layout":{"universe":1,"address":5,"gap":0}}),
    )
    .unwrap_err();
    assert!(error.contains("灯 1") && error.contains("灯 5") && error.contains("5–8"));
}
