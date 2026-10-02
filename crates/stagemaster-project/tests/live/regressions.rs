use super::support::*;
use serde_json::json;
use stagemaster_engine::live::{Error, Kind, LiveMixer};

#[test]
fn pause_resume_does_not_reassert_but_explicit_start_does_and_failed_publish_retains_intent() {
    let (doc, _, ids) = setup();
    let mut a = doc.compile_live_scene(&ids[0], 0).unwrap();
    let mut b = doc.compile_live_scene(&ids[1], 0).unwrap();
    let mut m = LiveMixer::new([1; 16], a.layout().clone(), 2).unwrap();
    let ah = open(&mut m, 1, Kind::Playback, 0);
    let bh = open(&mut m, 2, Kind::Playback, 0);
    a.start(0).unwrap();
    a.publish(&mut m, ah, 1).unwrap();
    b.start(0).unwrap();
    b.publish(&mut m, bh, 1).unwrap();
    a.pause(100).unwrap();
    a.publish(&mut m, ah, 2).unwrap();
    a.resume(200).unwrap();
    advance(&mut a, &mut m, ah, 3, 300);
    assert_eq!(values(&m)[1], 0x1fff);
    a.start(300).unwrap();
    assert_eq!(a.publish(&mut m, ah, 3), Err(Error::Sequence));
    assert_eq!(values(&m)[1], 0x1fff);
    a.publish(&mut m, ah, 4).unwrap();
    assert_eq!(values(&m)[1], 32768);
    advance(&mut b, &mut m, bh, 2, 400);
    assert_eq!(values(&m)[1], 32768);
    assert!(a.start(299).is_err());
    assert_eq!(values(&m)[1], 32768);
}
#[test]
fn unsaved_document_edits_with_same_revision_cannot_reuse_a_prepared_layout() {
    let (mut doc, fixture, ids) = setup();
    let mut scene = doc.compile_live_scene(&ids[0], 0).unwrap();
    let mut m = LiveMixer::new([1; 16], scene.layout().clone(), 2).unwrap();
    let h = open(&mut m, 1, Kind::Playback, 0);
    scene.start(0).unwrap();
    scene.publish(&mut m, h, 1).unwrap();
    let before = values(&m);
    set(&mut doc, &ids[0], &fixture, "red", 33_000);
    let mut changed = doc.compile_live_scene(&ids[0], 0).unwrap();
    assert_eq!(scene.revision_id, changed.revision_id);
    assert_ne!(scene.layout().id(), changed.layout().id());
    changed.start(0).unwrap();
    assert_eq!(changed.publish(&mut m, h, 2), Err(Error::Layout));
    assert_eq!(values(&m), before);
}
#[test]
fn released_or_disabled_effect_attributes_do_not_claim_defaults() {
    let (mut doc, fixture, ids) = setup();
    for original in doc.view().scenes[0].effects.clone() {
        let mut effect = serde_json::to_value(original).unwrap();
        effect["enabled"] = false.into();
        edit(
            &mut doc,
            json!({"op":"effect","command":{"kind":"put","sceneId":ids[0],"effect":effect}}),
        );
    }
    edit(
        &mut doc,
        json!({"op":"setSceneValue","sceneId":ids[0],"fixtureId":fixture,"attribute":"red","mode":"release","value":0}),
    );
    let mut a = doc.compile_live_scene(&ids[0], 0).unwrap();
    let mut b = doc.compile_live_scene(&ids[1], 0).unwrap();
    let mut m = LiveMixer::new([1; 16], a.layout().clone(), 2).unwrap();
    let bh = open(&mut m, 2, Kind::Playback, 0);
    let ah = open(&mut m, 1, Kind::Playback, 0);
    b.start(0).unwrap();
    b.publish(&mut m, bh, 1).unwrap();
    a.start(0).unwrap();
    a.publish(&mut m, ah, 1).unwrap();
    let mut out = [0; 7];
    let mut winners = [None; 7];
    m.render(&mut out, &mut winners).unwrap();
    assert_eq!(
        winners,
        [Some(bh), Some(bh), None, Some(ah), None, None, None]
    );
    assert_eq!(out, [20_000, 0x1fff, 0, 20 * 257, 0, 0, 32768]);
}
fn set(doc: &mut stagemaster_project::Document, scene: &str, fixture: &str, key: &str, value: u16) {
    edit(
        doc,
        json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"mode":"literal","value":value}),
    );
}

#[test]
fn prepared_encoder_rejects_equal_shape_but_changed_patch_without_touching_slots() {
    let (doc, _, ids) = setup();
    let scene = doc.compile_live_scene(&ids[0], 0).unwrap();
    let mut output = scene.prepare_output().unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["lighting"]["patches"][0]["address"] = 100.into();
    let changed =
        stagemaster_project::Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let mut moved = changed.compile_live_scene(&ids[0], 0).unwrap();
    assert_eq!(
        scene.layout().attributes().len(),
        moved.layout().attributes().len()
    );
    let mut mixer = LiveMixer::new([1; 16], moved.layout().clone(), 1).unwrap();
    let handle = open(&mut mixer, 1, Kind::Playback, 0);
    moved.start(0).unwrap();
    moved.publish(&mut mixer, handle, 1).unwrap();
    let mut slots = [77; 512];
    assert!(output.render(&mixer, &mut slots).is_err());
    assert_eq!(slots, [77; 512]);
    let mut correct = moved.prepare_output().unwrap();
    correct.render(&mixer, &mut slots).unwrap();
    assert!(slots[..99].iter().all(|v| *v == 0));
    assert_eq!(slots[102], 20);
}
