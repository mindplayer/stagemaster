use serde_json::{Value, json};
use stagemaster_playback::{LoopPlayback, LoopPlays};
use stagemaster_project::{Document, EditCommand};

fn audio(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Audio {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn loops(doc: &mut Document, command: Value) -> Result<(), String> {
    let mut wrapped = json!({"kind":"loopRegions"});
    wrapped["command"] = command;
    audio(doc, wrapped)
}
fn document() -> Document {
    let mut doc = Document::new("演出循环").unwrap();
    audio(
        &mut doc,
        json!({"kind":"setAsset","asset":{
            "digest":"ab".repeat(32),"fileName":"演出.wav","extension":"wav","durationMs":10000
        }}),
    )
    .unwrap();
    doc
}
fn add(doc: &mut Document, start: u64, end: u64) -> String {
    loops(
        doc,
        json!({"kind":"add","name":"等待台词","startMs":start,"endMs":end,
        "plays":{"kind":"count","count":3}}),
    )
    .unwrap();
    doc.audio_timeline()
        .unwrap()
        .loop_regions
        .iter()
        .find(|r| r.start_ms == start)
        .unwrap()
        .id
        .clone()
}
fn group(doc: &mut Document, ids: &[&str], action: Value) -> Result<(), String> {
    let mut wrapped = json!({"kind":"edit","ids":ids});
    wrapped["action"] = action;
    loops(doc, wrapped)
}

#[test]
fn named_sections_roundtrip_with_explicit_capability_and_legacy_omission() {
    let mut doc = document();
    let old: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert!(old["media"]["audioEditing"].get("loopRegions").is_none());
    let later = add(&mut doc, 3000, 4000);
    let first = add(&mut doc, 1000, 2000);
    group(
        &mut doc,
        &[&first],
        json!({"kind":"plays","plays":{"kind":"untilExit"}}),
    )
    .unwrap();
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    let track = doc.audio_timeline().unwrap();
    assert_eq!(track.loop_regions[0].id, first);
    assert_eq!(track.loop_regions[1].id, later);
    let saved: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert!(
        saved["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "media.audio-loop-regions")
    );
    group(&mut doc, &[&first, &later], json!({"kind":"remove"})).unwrap();
    assert!(doc.audio_timeline().unwrap().loop_regions.is_empty());
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    audio(&mut doc, json!({"kind":"clear"})).unwrap();
    let cleared: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert!(
        !cleared["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "media.audio-loop-regions")
    );
}

#[test]
fn copy_preserves_spacing_and_content_but_owns_identity_and_unlocks() {
    let mut doc = document();
    let a = add(&mut doc, 1000, 2000);
    let b = add(&mut doc, 3000, 4000);
    group(&mut doc, &[&b], json!({"kind":"enabled","enabled":false})).unwrap();
    group(&mut doc, &[&a, &b], json!({"kind":"locked","locked":true})).unwrap();
    let original = doc.audio_timeline().unwrap();
    group(
        &mut doc,
        &[&b, &a],
        json!({"kind":"copy","destinationMs":6000}),
    )
    .unwrap();
    let copied = doc.audio_timeline().unwrap();
    assert_eq!(&copied.loop_regions[..2], &original.loop_regions);
    for i in 0..2 {
        let source = &original.loop_regions[i];
        let copy = &copied.loop_regions[i + 2];
        assert_ne!(source.id, copy.id);
        assert_eq!(copy.start_ms, source.start_ms + 5000);
        assert_eq!(copy.end_ms, source.end_ms + 5000);
        assert_eq!(copy.plays, source.plays);
        assert_eq!(copy.enabled, source.enabled);
        assert!(!copy.locked);
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}

#[test]
fn group_operations_reject_mixed_locks_collisions_and_invalid_selections_atomically() {
    let mut doc = document();
    let a = add(&mut doc, 1000, 2000);
    let b = add(&mut doc, 3000, 4000);
    group(&mut doc, &[&b], json!({"kind":"locked","locked":true})).unwrap();
    for action in [
        json!({"kind":"remove"}),
        json!({"kind":"enabled","enabled":false}),
        json!({"kind":"plays","plays":{"kind":"untilExit"}}),
        json!({"kind":"move","destinationMs":5000}),
    ] {
        let before = doc.clone();
        assert!(
            group(&mut doc, &[&a, &b], action)
                .unwrap_err()
                .contains("锁定")
        );
        assert_eq!(doc, before);
    }
    group(&mut doc, &[&a, &b], json!({"kind":"locked","locked":false})).unwrap();
    for (ids, action) in [
        (vec![a.as_str(), a.as_str()], json!({"kind":"remove"})),
        (vec![], json!({"kind":"remove"})),
        (vec![a.as_str(), "unknown"], json!({"kind":"remove"})),
        (
            vec![a.as_str()],
            json!({"kind":"move","destinationMs":3500}),
        ),
        (
            vec![a.as_str(), b.as_str()],
            json!({"kind":"copy","destinationMs":8000}),
        ),
        (
            vec![a.as_str()],
            json!({"kind":"move","destinationMs":u64::MAX}),
        ),
        (
            vec![a.as_str()],
            json!({"kind":"plays","plays":{"kind":"count","count":0}}),
        ),
    ] {
        let before = doc.clone();
        assert!(group(&mut doc, &ids, action).is_err());
        assert_eq!(doc, before);
    }
    group(
        &mut doc,
        &[&b, &a],
        json!({"kind":"move","destinationMs":0}),
    )
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    assert_eq!(track.loop_regions[0].start_ms, 0);
    assert_eq!(track.loop_regions[1].start_ms, 2000);
}

#[test]
fn individual_edits_and_music_trim_preserve_all_data_on_error() {
    let mut doc = document();
    add(&mut doc, 5000, 8000);
    let original = serde_json::to_value(&doc.audio_timeline().unwrap().loop_regions[0]).unwrap();
    for (field, value) in [
        ("locked", json!(true)),
        ("enabled", json!(false)),
        ("name", json!("  ")),
        ("id", json!("a0000000-0000-4000-8000-000000000001")),
        ("startMs", json!(8000)),
        ("endMs", json!(10001)),
    ] {
        let mut region = original.clone();
        region[field] = value;
        let before = doc.clone();
        assert!(loops(&mut doc, json!({"kind":"put","region":region})).is_err());
        assert_eq!(doc, before);
    }
    let before = doc.clone();
    assert!(
        audio(&mut doc, json!({"kind":"trim","inMs":3000,"outMs":10000}))
            .unwrap_err()
            .contains("循环区段")
    );
    assert_eq!(doc, before);
    audio(&mut doc, json!({"kind":"trim","inMs":1000,"outMs":10000})).unwrap();
    assert_eq!(doc.audio_timeline().unwrap().loop_regions[0].start_ms, 5000);
}

#[test]
fn compiled_snapshot_filters_disabled_sections_and_uses_shared_integer_boundaries() {
    let mut doc = document();
    let a = add(&mut doc, 1, 10);
    let b = add(&mut doc, 10, 11);
    let adjacent = doc.audio_timeline().unwrap().compile_loops(44100).unwrap();
    assert_eq!(
        adjacent.schedule.regions()[0].end,
        adjacent.schedule.regions()[1].start
    );
    group(&mut doc, &[&b], json!({"kind":"enabled","enabled":false})).unwrap();
    let track = doc.audio_timeline().unwrap();
    let compiled = track.compile_loops(44100).unwrap();
    assert_eq!(compiled.region_ids, vec![a.clone()]);
    assert_eq!(compiled.schedule.duration(), 441_000);
    assert_eq!(compiled.schedule.regions()[0].start, 44);
    assert_eq!(compiled.schedule.regions()[0].end, 441);
    assert_eq!(compiled.schedule.regions()[0].plays, LoopPlays::Count(3));
    assert!(track.compile_loops(0).is_err());
    assert!(track.compile_loops(1).is_err()); // range disappears at this resolution
    let mut p = LoopPlayback::new(compiled.schedule, 44).unwrap();
    group(&mut doc, &[&a], json!({"kind":"remove"})).unwrap();
    // 1..10 ms is 397 frames after endpoint quantization, not floor(9 ms * 44100) = 396.
    assert_eq!(p.advance(397).unwrap().pass, Some(2)); // running snapshot remains immutable
}

#[test]
fn file_validation_rejects_missing_capability_duplicate_identity_and_unsupported_shapes() {
    let mut doc = document();
    add(&mut doc, 0, 100);
    let saved: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let reject =
        |value: &Value| assert!(Document::decode(&serde_json::to_vec(value).unwrap()).is_err());
    let mut missing = saved.clone();
    missing["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "media.audio-loop-regions");
    reject(&missing);
    let mut orphan = saved.clone();
    orphan.as_object_mut().unwrap().remove("media");
    reject(&orphan);
    for (field, value) in [
        ("id", saved["project"]["id"].clone()),
        ("name", json!("x".repeat(129))),
        ("plays", json!({"kind":"count","count":4_294_967_296_u64})),
        ("plays", json!({"kind":"untilExit","count":1})),
        ("plays", json!({"kind":"count","count":0})),
        ("extra", json!(true)),
    ] {
        let mut altered = saved.clone();
        altered["media"]["audioEditing"]["loopRegions"][0][field] = value;
        reject(&altered);
    }
    let mut repeated = saved;
    let mut duplicate = repeated["media"]["audioEditing"]["loopRegions"][0].clone();
    duplicate["startMs"] = json!(100);
    duplicate["endMs"] = json!(200);
    repeated["media"]["audioEditing"]["loopRegions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    reject(&repeated);
}

#[test]
fn capacity_is_enforced_before_a_group_copy_and_add() {
    let mut doc = document();
    let first = add(&mut doc, 0, 1);
    let mut value: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let template = value["media"]["audioEditing"]["loopRegions"][0].clone();
    for i in 1..128 {
        let mut region = template.clone();
        region["id"] = json!(format!("d0000000-0000-4000-8000-{i:012}"));
        region["startMs"] = json!(i * 2);
        region["endMs"] = json!(i * 2 + 1);
        value["media"]["audioEditing"]["loopRegions"]
            .as_array_mut()
            .unwrap()
            .push(region);
    }
    let mut full = Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    let before = full.clone();
    assert!(
        group(
            &mut full,
            &[&first],
            json!({"kind":"copy","destinationMs":500})
        )
        .is_err()
    );
    assert_eq!(full, before);
    assert!(
        loops(
            &mut full,
            json!({"kind":"add","name":"过多","startMs":500,"endMs":501,
        "plays":{"kind":"count","count":1}})
        )
        .is_err()
    );
    assert_eq!(full, before);
}
