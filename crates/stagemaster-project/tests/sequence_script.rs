use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
fn root() -> Value {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    root
}
fn decode(root: &Value) -> Result<Document, String> {
    Document::decode(&serde_json::to_vec(root).unwrap())
}
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    let mut operation = json!({"op":"sequence"});
    operation["command"] = command;
    doc.edit(serde_json::from_value(operation).unwrap())
}
fn script() -> Value {
    json!({"section":"第一幕 · 入场","trigger":"演员：现在开始。\n举手时执行。","notes":"等待掌声\n不要提前"})
}
fn set(doc: &mut Document, script: Value) -> Result<(), String> {
    let view = doc.view();
    let seq = &view.sequences[0];
    let mut command = json!({"kind":"updateStepScript","id":seq.id,"stepId":seq.steps[0].id});
    command["script"] = script;
    edit(doc, command)
}
#[test]
fn old_document_stays_plain_and_script_roundtrips_without_changing_timing() {
    let mut doc = decode(&root()).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&doc.encode().unwrap()).unwrap(),
        root()
    );
    let before = doc.compile_sequence(&doc.view().sequences[0].id).unwrap();
    set(&mut doc, script()).unwrap();
    let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
    assert_eq!(reopened, doc);
    let step = &reopened.view().sequences[0].steps[0];
    assert_eq!(
        step.script.as_ref().unwrap().trigger,
        "演员：现在开始。\n举手时执行。"
    );
    let after = reopened
        .compile_sequence(&reopened.view().sequences[0].id)
        .unwrap();
    assert_eq!(before.plan, after.plan);
    assert!(before.steps[0].script.is_none());
    assert_eq!(after.steps[0].script, step.script);
    let (mut a, mut b) = (Player::new(before.plan, 0), Player::new(after.plan, 0));
    a.execute(0, 0).unwrap();
    b.execute(0, 0).unwrap();
    for t in (0..10000).step_by(17) {
        a.advance(t).unwrap();
        b.advance(t).unwrap();
        assert_eq!(a.values(), b.values());
    }
}
#[test]
fn loaded_metadata_and_device_plan_are_independent_of_later_notes() {
    let mut doc = decode(&root()).unwrap();
    set(&mut doc, script()).unwrap();
    let id = doc.view().sequences[0].id.clone();
    let loaded = doc.compile_sequence(&id).unwrap();
    set(
        &mut doc,
        json!({"section":"第二幕","trigger":"新提示","notes":""}),
    )
    .unwrap();
    assert_eq!(
        loaded.steps[0].script.as_ref().unwrap().section,
        "第一幕 · 入场"
    );
    let built = doc
        .build_package(&[PackageSelection::Sequence { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    let program = archive.load(built.bytes.as_slice(), 0).unwrap();
    assert_eq!(loaded.plan, program.plan);
    assert!(
        !built
            .bytes
            .windows("新提示".len())
            .any(|w| w == "新提示".as_bytes())
    );
}
#[test]
fn invalid_metadata_is_atomic_and_capability_is_required_on_load() {
    let mut doc = decode(&root()).unwrap();
    for (field, text) in [
        ("section", "字".repeat(81)),
        ("trigger", "字".repeat(1025)),
        ("notes", "字".repeat(4097)),
        ("notes", "\u{0000}".into()),
        ("trigger", "\u{0085}".into()),
    ] {
        let before = doc.clone();
        let mut bad = script();
        bad[field] = text.into();
        assert!(set(&mut doc, bad).is_err());
        assert_eq!(doc, before);
    }
    let mut raw = root();
    raw["lighting"]["sequences"][0]["steps"][0]["script"] = script();
    assert!(decode(&raw).unwrap_err().contains("剧本提示能力"));
    raw["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"lighting.sequence-script","version":1}));
    assert!(decode(&raw).is_ok());
    raw["lighting"]["sequences"][0]["steps"][0]["script"]["unexpected"] = true.into();
    assert!(decode(&raw).is_err());
}
#[test]
fn copy_move_old_updates_and_delete_preserve_or_clean_script() {
    let mut doc = decode(&root()).unwrap();
    set(&mut doc, script()).unwrap();
    let v = doc.view();
    let seq = &v.sequences[0];
    let first = &seq.steps[0];
    edit(&mut doc,json!({"kind":"updateStep","id":seq.id,"stepId":first.id,"name":"改名","number":"1","sceneId":first.scene_id,"delayMs":0,"fadeMs":500,"waitMs":null})).unwrap();
    assert_eq!(doc.view().sequences[0].steps[0].script, first.script);
    edit(
        &mut doc,
        json!({"kind":"duplicateStep","id":seq.id,"stepId":first.id}),
    )
    .unwrap();
    let copy = doc.view().sequences[0].steps[1].id.clone();
    assert_eq!(doc.view().sequences[0].steps[1].script, first.script);
    set(&mut doc, Value::Null).unwrap();
    assert!(
        String::from_utf8(doc.encode().unwrap())
            .unwrap()
            .contains("lighting.sequence-script")
    );
    edit(
        &mut doc,
        json!({"kind":"moveStep","id":seq.id,"stepId":copy,"index":2}),
    )
    .unwrap();
    assert_eq!(doc.view().sequences[0].steps[2].script, first.script);
    edit(
        &mut doc,
        json!({"kind":"removeStep","id":seq.id,"stepId":copy}),
    )
    .unwrap();
    assert!(
        !String::from_utf8(doc.encode().unwrap())
            .unwrap()
            .contains("lighting.sequence-script")
    );
}
#[test]
fn unicode_bounds_and_total_bytes_are_bounded_without_truncation() {
    let mut doc = decode(&root()).unwrap();
    set(
        &mut doc,
        json!({"section":"🎭".repeat(80),"trigger":"","notes":""}),
    )
    .unwrap();
    set(&mut doc, json!({"section":" \t","trigger":"\n","notes":""})).unwrap();
    assert!(doc.view().sequences[0].steps[0].script.is_none());
    set(
        &mut doc,
        json!({"section":"","trigger":"","notes":"字".repeat(4096)}),
    )
    .unwrap();
    for _ in 0..4 {
        let v = doc.view();
        edit(&mut doc,json!({"kind":"duplicateStep","id":v.sequences[0].id,"stepId":v.sequences[0].steps[0].id})).unwrap();
    }
    let before = doc.clone();
    let v = doc.view();
    assert!(edit(&mut doc,json!({"kind":"duplicateStep","id":v.sequences[0].id,"stepId":v.sequences[0].steps[0].id})).unwrap_err().contains("64 KiB"));
    assert_eq!(doc, before);
}
