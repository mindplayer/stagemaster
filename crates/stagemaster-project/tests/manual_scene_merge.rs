#[allow(dead_code)]
#[path = "support/fixture_function.rs"]
mod support;
use serde_json::json;
use sha2::{Digest, Sha256};
use stagemaster_project::{Document, ManualSceneCapture, ManualSceneReading};
fn capture(document: &Document, fixture: &str, values: &[(&str, u16)]) -> ManualSceneCapture {
    document
        .capture_manual_scene(
            &format!("{:x}", Sha256::digest(document.encode().unwrap())),
            values
                .iter()
                .map(|(a, value)| ManualSceneReading {
                    fixture_id: fixture.into(),
                    attribute: (*a).into(),
                    value: *value,
                })
                .collect(),
        )
        .unwrap()
}
fn set(document: &mut Document, scenes: &str, fixture: &str, a: &str, value: u16) {
    support::edit(document, json!({"op":"setSceneValue","sceneId":scenes,"fixtureId":fixture,"attribute":a,"mode":"literal","value":value})).unwrap();
}
#[test]
fn sparse_merge_preserves_effects_other_values_identity_references_and_roundtrips() {
    let (mut document, fixture, scenes) = support::setup(false);
    set(&mut document, &scenes[0], &fixture, "dimmer", 30000);
    support::choose(&mut document, &scenes[0], &fixture, "gobo-wheel", "dots", 0).unwrap();
    support::edit(
        &mut document,
        json!({"op":"effect","command":{"kind":"put","sceneId":scenes[0],"effect":{
        "id":uuid::Uuid::new_v4().to_string(),"name":"呼吸","enabled":true,"fixtureIds":[fixture],"periodMs":2000,
        "spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"smooth","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":1000,"high":50000}]}}}),
    )
    .unwrap();
    support::edit(
        &mut document,
        json!({"op":"sequence","command":{"kind":"add","name":"列表","sceneId":scenes[0]}}),
    )
    .unwrap();
    let mut raw = support::raw(&document);
    raw["lighting"]["scenes"][0]["assignments"]
        .as_array_mut()
        .unwrap()
        .retain(|a| a["target"]["attribute"] != "color-wheel");
    document = support::decode(&raw).unwrap();
    let before = support::raw(&document);
    let captured = capture(&document, &fixture, &[("dimmer", 0), ("color-wheel", 5140)]);
    let merge = document
        .prepare_manual_scene_merge(&captured, &scenes[0])
        .unwrap();
    let review = merge.summary();
    assert_eq!(
        (
            review.added,
            review.replaced,
            review.unchanged,
            review.preserved,
            review.effects
        ),
        (1, 1, 0, 3, 1)
    );
    assert_eq!(review.rows[0].previous_value, Some(30000));
    assert_eq!(review.rows[0].effect_names, vec!["呼吸"]);
    document.merge_manual_scene(&merge).unwrap();
    let after = support::raw(&document);
    for key in ["id", "name", "effects"] {
        assert_eq!(
            before["lighting"]["scenes"][0][key],
            after["lighting"]["scenes"][0][key]
        );
    }
    assert_eq!(
        before["lighting"]["scenes"][0]["assignments"][1],
        after["lighting"]["scenes"][0]["assignments"][1]
    );
    assert_eq!(
        before["lighting"]["scenes"][1],
        after["lighting"]["scenes"][1]
    );
    assert_eq!(
        before["lighting"]["sequences"],
        after["lighting"]["sequences"]
    );
    let value = document.view();
    assert_eq!(value.scenes[0].values[0].value, Some(0));
    assert_eq!(
        value.scenes[0]
            .values
            .iter()
            .find(|value| value.attribute == "color-wheel")
            .unwrap()
            .function_value
            .as_ref()
            .unwrap()
            .function_key,
        "red"
    );
    assert_eq!(
        document,
        Document::decode(&document.encode().unwrap()).unwrap()
    );
    document.compile_scene(&scenes[0]).unwrap();
}
#[test]
fn updating_linked_value_detaches_only_that_assignment_and_checks_review_dependencies() {
    let (mut document, fixture, scenes) = support::setup(false);
    set(&mut document, &scenes[0], &fixture, "dimmer", 30000);
    support::edit(&mut document,json!({"op":"library","command":{"kind":"recordPreset","name":"亮度预设","sceneId":scenes[0],"fixtureIds":[fixture],"attributes":["dimmer"]}})).unwrap();
    let preset = document.view().presets[0].id.clone();
    for scene in &scenes {
        support::edit(&mut document,json!({"op":"library","command":{"kind":"applyPreset","id":preset,"sceneId":scene,"fixtureIds":[fixture],"attributes":["dimmer"],"linked":true}})).unwrap();
    }
    let original = document.clone();
    let captured = capture(&document, &fixture, &[("dimmer", 30000)]);
    let merge = document
        .prepare_manual_scene_merge(&captured, &scenes[0])
        .unwrap();
    assert_eq!(merge.summary().replaced, 1); // Equal output still explicitly removes the link.
    assert_eq!(
        merge.summary().rows[0].previous_preset.as_deref(),
        Some("亮度预设")
    );
    document.merge_manual_scene(&merge).unwrap();
    assert!(document.view().scenes[0].values[0].preset_id.is_none());
    assert!(document.view().scenes[1].values[0].preset_id.is_some());
    assert_eq!(
        support::raw(&document)["lighting"]["presets"],
        support::raw(&original)["lighting"]["presets"]
    );
    let mut changed = original;
    support::edit(
        &mut changed,
        json!({"op":"library","command":{"kind":"renamePreset","id":preset,"name":"另一预设名"}}),
    )
    .unwrap();
    let before = changed.clone();
    assert!(changed.merge_manual_scene(&merge).is_err());
    assert_eq!(changed, before);
}
#[test]
fn unchanged_is_noop_and_changed_or_missing_target_rejects_atomically() {
    let (mut document, fixture, scenes) = support::setup(false);
    set(&mut document, &scenes[0], &fixture, "dimmer", 0);
    let captured = capture(&document, &fixture, &[("dimmer", 0)]);
    let merge = document
        .prepare_manual_scene_merge(&captured, &scenes[0])
        .unwrap();
    assert_eq!(merge.summary().unchanged, 1);
    let before = document.clone();
    document.merge_manual_scene(&merge).unwrap();
    assert_eq!(document, before);
    assert!(
        document
            .prepare_manual_scene_merge(&captured, "missing")
            .is_err()
    );
    set(&mut document, &scenes[0], &fixture, "dimmer", 60000);
    let before = document.clone();
    assert!(document.merge_manual_scene(&merge).is_err());
    assert_eq!(document, before);
    let mut other = Document::new("另一个工程").unwrap();
    let before = other.clone();
    assert!(other.merge_manual_scene(&merge).is_err());
    assert_eq!(other, before);
}
#[test]
fn release_is_replaced_and_function_ranges_remain_exact_in_both_encodings() {
    for fine in [false, true] {
        let (mut document, fixture, scenes) = support::setup(fine);
        support::edit(&mut document,json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":"dimmer","mode":"release","value":0})).unwrap();
        let color = if fine { 20 } else { 5140 };
        let captured = capture(
            &document,
            &fixture,
            &[("dimmer", 0), ("color-wheel", color), ("shutter", 37008)],
        );
        let merge = document
            .prepare_manual_scene_merge(&captured, &scenes[0])
            .unwrap();
        assert_eq!(merge.summary().rows[0].previous_mode, "release");
        assert_eq!(merge.summary().rows[0].previous_value, None);
        document.merge_manual_scene(&merge).unwrap();
        let values = &document.view().scenes[0].values;
        assert_eq!(
            values
                .iter()
                .find(|value| value.attribute == "color-wheel")
                .unwrap()
                .value,
            Some(u64::from(color))
        );
        assert_eq!(
            values
                .iter()
                .find(|value| value.attribute == "shutter")
                .unwrap()
                .value,
            Some(37008)
        );
        document.compile_scene(&scenes[0]).unwrap();
    }
}
