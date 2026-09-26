use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Library {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn setup() -> (Document, Vec<String>, Vec<String>) {
    let mut doc = Document::new("资源验证").unwrap();
    let view = doc.view();
    for index in 0..3 {
        doc.edit(EditCommand::AddFixture {
            name: format!("灯具 {index}"),
            profile_id: view.profiles[usize::from(index < 2)].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address: index * 4 + 1,
        })
        .unwrap();
    }
    for name in ["来源", "引用场景", "独立场景"] {
        doc.edit(EditCommand::AddScene { name: name.into() })
            .unwrap();
    }
    let view = doc.view();
    (
        doc,
        view.fixtures.into_iter().map(|f| f.id).collect(),
        view.scenes.into_iter().map(|s| s.id).collect(),
    )
}
fn set(doc: &mut Document, scene: &str, fixture: &str, attr: &str, value: u16) {
    doc.edit(EditCommand::SetSceneValue {
        scene_id: scene.into(),
        fixture_id: fixture.into(),
        attribute: attr.into(),
        mode: stagemaster_project::ValueMode::Literal,
        value,
    })
    .unwrap();
}
fn record(doc: &mut Document, scene: &str, fixtures: &[String], attrs: &[&str]) -> String {
    edit(doc, json!({"kind":"recordPreset","name":"常用颜色","sceneId":scene,"fixtureIds":fixtures,"attributes":attrs})).unwrap();
    doc.view().presets.last().unwrap().id.clone()
}
fn assignment(doc: &Document, scene: &str, fixture: &str, attr: &str) -> Value {
    let view = serde_json::to_value(doc.view()).unwrap();
    view["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == scene)
        .unwrap()["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["fixtureId"] == fixture && v["attribute"] == attr)
        .unwrap()
        .clone()
}
fn reject(doc: &mut Document, command: Value, message: &str) {
    let before = doc.encode().unwrap();
    let error = edit(doc, command).unwrap_err();
    assert!(error.contains(message), "{error}");
    assert_eq!(doc.encode().unwrap(), before);
}
#[test]
fn ordered_groups_round_trip_duplicate_and_do_not_rewrite_scenes() {
    let (mut doc, f, _) = setup();
    let scenes = serde_json::to_value(doc.view().scenes).unwrap();
    edit(
        &mut doc,
        json!({"kind":"saveGroup","id":null,"name":"反向","fixtureIds":[f[1],f[0]]}),
    )
    .unwrap();
    let group = doc.view().groups[0].id.clone();
    edit(
        &mut doc,
        json!({"kind":"duplicate","resource":"group","id":group,"name":"复制"}),
    )
    .unwrap();
    assert_ne!(doc.view().groups[0].id, doc.view().groups[1].id);
    edit(
        &mut doc,
        json!({"kind":"saveGroup","id":group,"name":"单灯","fixtureIds":[f[2]]}),
    )
    .unwrap();
    assert_eq!(
        doc.view().groups[1].fixture_ids,
        vec![f[1].clone(), f[0].clone()]
    );
    edit(
        &mut doc,
        json!({"kind":"remove","resource":"group","id":group,"keepValues":false}),
    )
    .unwrap();
    assert_eq!(serde_json::to_value(doc.view().scenes).unwrap(), scenes);
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn invalid_or_empty_group_members_and_stale_ids_are_atomic() {
    let (mut doc, f, _) = setup();
    for members in [json!([]), json!([f[0], f[0]])] {
        reject(
            &mut doc,
            json!({"kind":"saveGroup","id":null,"name":"组","fixtureIds":members}),
            "不重复",
        );
    }
    reject(
        &mut doc,
        json!({"kind":"saveGroup","id":null,"name":"组","fixtureIds":["missing"]}),
        "不存在",
    );
    reject(
        &mut doc,
        json!({"kind":"duplicate","resource":"group","id":"missing","name":"复制"}),
        "不存在",
    );
}
#[test]
fn capture_mask_resolves_links_and_skips_release_without_inventing_values() {
    let (mut doc, f, s) = setup();
    set(&mut doc, &s[0], &f[0], "red", 12345);
    doc.edit(EditCommand::SetSceneValue {
        scene_id: s[0].clone(),
        fixture_id: f[1].clone(),
        attribute: "red".into(),
        mode: stagemaster_project::ValueMode::Release,
        value: 0,
    })
    .unwrap();
    let p = record(&mut doc, &s[0], &f[..2], &["red"]);
    assert_eq!(doc.view().presets[0].values.len(), 1);
    edit(&mut doc,json!({"kind":"applyPreset","id":p,"sceneId":s[1],"fixtureIds":f,"attributes":["red"],"linked":true})).unwrap();
    let p2 = record(&mut doc, &s[1], &f[..1], &["red"]);
    assert_ne!(p, p2);
    assert_eq!(doc.view().presets[1].values[0].value, Some(12345));
    reject(
        &mut doc,
        json!({"kind":"recordPreset","name":"空","sceneId":s[0],"fixtureIds":[f[1]],"attributes":["red"]}),
        "没有已记录",
    );
}
#[test]
fn preset_update_propagates_only_links_and_compile_snapshots_are_immutable() {
    let (mut doc, f, s) = setup();
    set(&mut doc, &s[0], &f[0], "red", 1000);
    let p = record(&mut doc, &s[0], &f[..1], &["red"]);
    for (scene, linked) in [(&s[1], true), (&s[2], false)] {
        edit(&mut doc,json!({"kind":"applyPreset","id":p,"sceneId":scene,"fixtureIds":[f[0]],"attributes":["red"],"linked":linked})).unwrap();
    }
    doc.edit(
        serde_json::from_value(
            json!({"op":"sequence","command":{"kind":"add","name":"演出","sceneId":s[1]}}),
        )
        .unwrap(),
    )
    .unwrap();
    let sequence = doc.view().sequences[0].id.clone();
    let old = doc.compile_sequence(&sequence).unwrap();
    set(&mut doc, &s[0], &f[0], "red", 60000);
    edit(&mut doc,json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["red"],"mode":"existing"})).unwrap();
    assert_eq!(assignment(&doc, &s[1], &f[0], "red")["value"], 60000);
    assert_eq!(assignment(&doc, &s[2], &f[0], "red")["value"], 1000);
    assert_eq!(old.plan.steps()[0].target[1], 1000);
    assert_eq!(
        doc.compile_sequence(&sequence).unwrap().plan.steps()[0].target[1],
        60000
    );
    assert_eq!(doc.view().presets[0].used_by_scenes.len(), 1);
    assert_eq!(doc.view().presets[0].used_by_sequences[0].id, sequence);
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn updates_distinguish_existing_merge_replace_and_protect_removed_targets() {
    let (mut doc, f, s) = setup();
    let p = record(&mut doc, &s[0], &f[..1], &["red"]);
    edit(&mut doc,json!({"kind":"applyPreset","id":p,"sceneId":s[1],"fixtureIds":[f[0]],"attributes":["red"],"linked":true})).unwrap();
    edit(&mut doc,json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["red","blue"],"mode":"existing"})).unwrap();
    assert_eq!(doc.view().presets[0].values.len(), 1);
    edit(&mut doc,json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["red","blue"],"mode":"merge"})).unwrap();
    assert_eq!(doc.view().presets[0].values.len(), 2);
    assert_eq!(assignment(&doc, &s[1], &f[0], "blue")["mode"], "literal");
    reject(
        &mut doc,
        json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["blue"],"mode":"replace"}),
        "引用场景",
    );
    edit(&mut doc,json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["red"],"mode":"replace"})).unwrap();
    assert_eq!(doc.view().presets[0].values.len(), 1);
    reject(
        &mut doc,
        json!({"kind":"updatePreset","id":p,"sceneId":s[0],"fixtureIds":[f[1]],"attributes":["red"],"mode":"existing"}),
        "没有交集",
    );
}
#[test]
fn detach_and_delete_keep_values_preserve_scene_results_and_identity() {
    let (mut doc, f, s) = setup();
    set(&mut doc, &s[0], &f[0], "red", 42000);
    let p = record(&mut doc, &s[0], &f[..1], &["red"]);
    for scene in &s[1..] {
        edit(&mut doc,json!({"kind":"applyPreset","id":p,"sceneId":scene,"fixtureIds":[f[0]],"attributes":["red"],"linked":true})).unwrap();
    }
    edit(
        &mut doc,
        json!({"kind":"renamePreset","id":p,"name":"新名"}),
    )
    .unwrap();
    assert_eq!(assignment(&doc, &s[1], &f[0], "red")["presetId"], p);
    edit(
        &mut doc,
        json!({"kind":"duplicate","resource":"preset","id":p,"name":"新名"}),
    )
    .unwrap();
    assert!(doc.view().presets[1].used_by_scenes.is_empty());
    reject(
        &mut doc,
        json!({"kind":"remove","resource":"preset","id":p,"keepValues":false}),
        "引用场景",
    );
    edit(
        &mut doc,
        json!({"kind":"detach","sceneId":s[1],"fixtureIds":[f[0]],"attributes":["red"]}),
    )
    .unwrap();
    edit(
        &mut doc,
        json!({"kind":"remove","resource":"preset","id":p,"keepValues":true}),
    )
    .unwrap();
    for scene in &s[1..] {
        let a = assignment(&doc, scene, &f[0], "red");
        assert_eq!(a["value"], 42000);
        assert_eq!(a["mode"], "literal");
        assert!(a["presetId"].is_null());
    }
}
#[test]
fn copy_is_literal_masked_and_rejects_incompatible_targets_without_partial_change() {
    let (mut doc, f, s) = setup();
    set(&mut doc, &s[0], &f[0], "red", 54321);
    let p = record(&mut doc, &s[0], &f[..1], &["red"]);
    edit(&mut doc,json!({"kind":"applyPreset","id":p,"sceneId":s[0],"fixtureIds":[f[0]],"attributes":["red"],"linked":true})).unwrap();
    reject(
        &mut doc,
        json!({"kind":"copyValues","sceneId":s[0],"sourceId":f[0],"fixtureIds":[f[1],f[2]],"attributes":["red"]}),
        "没有这个属性",
    );
    edit(&mut doc,json!({"kind":"copyValues","sceneId":s[0],"sourceId":f[0],"fixtureIds":[f[1]],"attributes":["red"]})).unwrap();
    let a = assignment(&doc, &s[0], &f[1], "red");
    assert_eq!(a["value"], 54321);
    assert_eq!(a["mode"], "literal");
    assert_eq!(assignment(&doc, &s[0], &f[1], "dimmer")["value"], 0);
}
#[test]
fn invalid_batch_after_library_edit_rolls_back_all_resources() {
    let (mut doc, f, _) = setup();
    let before = doc.clone();
    let command = json!({"op":"batch","commands":[
        {"op":"library","command":{"kind":"saveGroup","id":null,"name":"组","fixtureIds":f}},
        {"op":"library","command":{"kind":"remove","resource":"preset","id":"missing","keepValues":true}}
    ]});
    assert!(doc.edit(serde_json::from_value(command).unwrap()).is_err());
    assert_eq!(doc, before);
}
