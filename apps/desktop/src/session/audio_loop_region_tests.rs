use super::*;
use serde_json::{Value, json};

fn edit(command: Value) -> EditCommand {
    EditCommand::Audio {
        command: stagemaster_project::AudioEdit::LoopRegions {
            command: serde_json::from_value(command).unwrap(),
        },
    }
}

#[test]
fn loop_group_history_is_atomic_and_noop_or_failure_preserves_generation_and_redo() {
    let mut doc = Document::new("循环历史").unwrap();
    doc.edit(
        serde_json::from_value(json!({"op":"audio","command":{"kind":"setAsset","asset":{
            "digest":"ab".repeat(32),"fileName":"演出.wav","extension":"wav","durationMs":10000
        }}}))
        .unwrap(),
    )
    .unwrap();
    for start in [1000, 3000] {
        doc.edit(edit(
            json!({"kind":"add","name":"区段","startMs":start,"endMs":start+1000,
            "plays":{"kind":"count","count":2}}),
        ))
        .unwrap();
    }
    let ids: Vec<_> = doc
        .audio_timeline()
        .unwrap()
        .loop_regions
        .into_iter()
        .map(|r| r.id)
        .collect();
    let command = |kind: &str, destination: u64| {
        edit(json!({"kind":"edit","ids":ids,
        "action":{"kind":kind,"destinationMs":destination}}))
    };
    let mut s = Session::default();
    s.replace(doc.clone(), None);
    let generation = s.generation;
    s.edit(generation, command("move", 1000)).unwrap();
    assert!(s.undo.is_empty());
    assert_eq!(s.generation, generation);
    s.edit(generation, command("copy", 6000)).unwrap();
    assert_eq!(s.undo.len(), 1);
    let after = s.document.clone();
    s.history(s.generation, false).unwrap();
    assert_eq!(s.document.as_ref(), Some(&doc));
    let generation = s.generation;
    assert!(s.edit(generation, command("copy", 1500)).is_err());
    assert_eq!(s.document.as_ref(), Some(&doc));
    assert_eq!(s.generation, generation);
    assert_eq!(s.redo.len(), 1);
    s.history(generation, true).unwrap();
    assert_eq!(s.document, after);
    let bytes = s.document.as_ref().unwrap().encode().unwrap();
    assert_eq!(
        Document::decode(&bytes).unwrap(),
        *s.document.as_ref().unwrap()
    );
}

#[test]
fn linear_only_test_loader_rejects_formal_loops_and_preserves_the_old_voice() {
    let mut doc = Document::new("循环准备保护").unwrap();
    doc.edit(
        serde_json::from_value(json!({"op":"audio","command":{"kind":"setAsset","asset":{
            "digest":"ab".repeat(32),"fileName":"演出.wav","extension":"wav","durationMs":10000
        }}}))
        .unwrap(),
    )
    .unwrap();
    let mut preview = crate::audio::AudioPreview::default();
    preview
        .load("not-opened.wav".into(), doc.audio_timeline().unwrap())
        .unwrap();
    doc.edit(edit(
        json!({"kind":"add","name":"等待","startMs":0,"endMs":1000,
        "plays":{"kind":"untilExit"}}),
    ))
    .unwrap();
    assert!(
        preview
            .load("not-opened.wav".into(), doc.audio_timeline().unwrap())
            .unwrap_err()
            .contains("演出循环")
    );
    assert!(preview.active()); // failed load preserves the old, playable track
    preview.synchronize(&doc);
    assert!(!preview.active()); // editing an enabled loop cannot keep the old linear player
}
