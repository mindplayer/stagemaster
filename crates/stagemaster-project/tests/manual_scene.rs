#[allow(dead_code)]
#[path = "support/fixture_function.rs"]
mod support;
use serde_json::json;
use sha2::{Digest, Sha256};
use stagemaster_project::{Document, ManualSceneReading as Reading};
fn identity(doc: &Document) -> String {
    format!("{:x}", Sha256::digest(doc.encode().unwrap()))
}
fn reading(fixture: &str, attribute: &str, value: u16) -> Reading {
    Reading {
        fixture_id: fixture.into(),
        attribute: attribute.into(),
        value,
    }
}
#[test]
fn full_single_universe_record_is_bounded_and_preserves_all_512_values() {
    let doc = Document::new("全通道录入").unwrap();
    let view = doc.view();
    let mut raw = support::raw(&doc);
    let mut fixtures = Vec::new();
    let mut patches = Vec::new();
    let mut readings = Vec::new();
    for i in 0..512_u16 {
        let id = uuid::Uuid::new_v4().to_string();
        fixtures.push(json!({"id":id,"name":format!("灯 {i}"),"profileId":view.profiles[0].id,"domainId":view.domains[0].id}));
        patches
            .push(json!({"fixtureId":id,"domainId":view.domains[0].id,"universe":1,"address":i+1}));
        readings.push(reading(&id, "dimmer", i * 128));
    }
    raw["lighting"]["fixtures"] = fixtures.into();
    raw["lighting"]["patches"] = patches.into();
    let mut doc = support::decode(&raw).unwrap();
    let captured = doc.capture_manual_scene(&identity(&doc), readings).unwrap();
    let scene = doc.record_manual_scene(&captured, "全通道记录").unwrap();
    let compiled = doc.compile_scene(&scene).unwrap();
    assert_eq!(compiled.plan.steps()[0].target.len(), 512);
    for (i, value) in compiled.plan.steps()[0].target.iter().enumerate() {
        assert_eq!(*value, u16::try_from(i).unwrap() * 128);
    }
}
#[test]
fn sparse_recording_roundtrips_zero_and_functions_without_filling_other_attributes() {
    for fine in [false, true] {
        let (mut doc, f, _) = support::setup(fine);
        let before = doc.clone();
        let capture = doc
            .capture_manual_scene(
                &identity(&doc),
                vec![
                    reading(&f, "dimmer", 0),
                    reading(&f, "color-wheel", if fine { 20 } else { 20 * 257 }),
                    reading(&f, "shutter", 144 * 257),
                ],
            )
            .unwrap();
        assert_eq!(doc, before);
        let id = doc.record_manual_scene(&capture, "现场记录").unwrap();
        let scene = doc.view().scenes.into_iter().find(|s| s.id == id).unwrap();
        assert_eq!(scene.values.len(), 3);
        assert!(scene.effects.is_empty());
        assert_eq!(scene.values[0].value, Some(0));
        assert_eq!(
            scene.values[1]
                .function_value
                .as_ref()
                .unwrap()
                .function_key,
            "red"
        );
        let compiled = doc.compile_scene(&id).unwrap();
        for r in capture.readings() {
            let scene_value = scene
                .values
                .iter()
                .find(|v| v.attribute == r.attribute)
                .unwrap();
            let typed = match &scene_value.function_value {
                Some(value) => stagemaster_project::ProfileDefault::Function(value.clone()),
                None => stagemaster_project::ProfileDefault::Normalized(
                    u16::try_from(scene_value.value.unwrap()).unwrap(),
                ),
            };
            let (_, value) = compiled
                .output
                .manual_value(&f, &r.attribute, Some(&typed))
                .unwrap();
            assert_eq!(value, Some(r.value));
        }
        assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    }
}
#[test]
fn bad_capture_and_record_are_atomic_and_never_guess_missing_or_changed_definitions() {
    let (mut doc, f, _) = support::setup(false);
    let layout = identity(&doc);
    for values in [
        vec![],
        vec![reading(&f, "dimmer", 0); 513],
        vec![reading(&f, "missing", 0)],
        vec![reading("missing", "dimmer", 0)],
        vec![reading(&f, "color-wheel", 21 * 257)],
        vec![reading(&f, "shutter", 30)],
        vec![reading(&f, "dimmer", 0); 2],
    ] {
        assert!(doc.capture_manual_scene(&layout, values).is_err());
    }
    assert!(
        doc.capture_manual_scene("bad", vec![reading(&f, "dimmer", 0)])
            .is_err()
    );
    let capture = doc
        .capture_manual_scene(&layout, vec![reading(&f, "dimmer", 0)])
        .unwrap();
    let before = doc.clone();
    for name in ["", "入场", "bad\nname"] {
        assert!(doc.record_manual_scene(&capture, name).is_err());
        assert_eq!(doc, before);
    }
    let mut other = Document::new("其他工程").unwrap();
    assert!(other.record_manual_scene(&capture, "记录").is_err());
    support::edit(
        &mut doc,
        json!({"op":"setInfo","name":"改名","description":"无关内容"}),
    )
    .unwrap();
    assert!(capture.check(&doc).is_ok());
    support::edit(
        &mut doc,
        json!({"op":"updateFixture","id":f,"name":"改灯名","universe":1,"address":1}),
    )
    .unwrap();
    assert!(capture.check(&doc).is_ok());
    support::edit(
        &mut doc,
        json!({"op":"updateFixture","id":f,"name":"改配适","universe":1,"address":20}),
    )
    .unwrap();
    let changed = doc.clone();
    assert!(doc.record_manual_scene(&capture, "记录").is_err());
    assert_eq!(doc, changed);
    let mut value = support::raw(&before);
    let profiles = value["lighting"]["profiles"].as_array_mut().unwrap();
    profiles.last_mut().unwrap()["channels"][1]["functions"][1]["dmxDefault"] = 22.into();
    assert!(capture.check(&support::decode(&value).unwrap()).is_err());
}
