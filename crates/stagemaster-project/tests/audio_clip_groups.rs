use serde_json::{Value, json};
use stagemaster_project::{AudioLightingClip, Document, EditCommand};
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Audio {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn fixture() -> Document {
    let mut doc = Document::new("片段组").unwrap();
    let v = doc.view();
    doc.edit(serde_json::from_value(json!({"op":"addFixture","name":"测试灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
    doc.edit(serde_json::from_value(json!({"op":"addScene","name":"亮场"})).unwrap())
        .unwrap();
    let scene = doc.view().scenes[0].id.clone();
    edit(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":20000}})).unwrap();
    edit(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    for (start, end) in [(1000, 2000), (3000, 4500), (7000, 8000)] {
        edit(&mut doc,json!({"kind":"addLightingClip","name":format!("片段{start}"),"sceneId":scene,"startMs":start,"endMs":end,"fadeMs":500})).unwrap();
    }
    doc
}
fn clips(doc: &Document) -> Vec<AudioLightingClip> {
    doc.audio_timeline().unwrap().lighting_clips.unwrap()
}
fn group(doc: &mut Document, ids: &[String], action: Value) -> Result<(), String> {
    let mut command = json!({"kind":"editLightingClips","ids":ids});
    command["action"] = action;
    edit(doc, command)
}
#[test]
fn unordered_move_preserves_intervals_gaps_identities_and_unselected_data() {
    let mut doc = fixture();
    let before = clips(&doc);
    group(
        &mut doc,
        &[before[1].id.clone(), before[0].id.clone()],
        json!({"kind":"move","destinationMs":8000}),
    )
    .unwrap();
    let after = clips(&doc);
    assert_eq!(after[0], before[2]); // adjacency is legal
    for i in 0..2 {
        let mut expected = before[i].clone();
        expected.start_ms += 7000;
        expected.end_ms += 7000;
        assert_eq!(after[i + 1], expected);
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    group(
        &mut doc,
        &[before[0].id.clone(), before[1].id.clone()],
        json!({"kind":"move","destinationMs":0}),
    )
    .unwrap();
    assert_eq!(clips(&doc)[0].start_ms, 0);
}
#[test]
fn locked_source_copy_is_independent_and_remove_restores_original() {
    let mut doc = fixture();
    let old = clips(&doc);
    edit(
        &mut doc,
        json!({"kind":"setLightingClipLock","id":old[0].id,"locked":true}),
    )
    .unwrap();
    let before = doc.clone();
    let ids = vec![old[1].id.clone(), old[0].id.clone()];
    for action in [
        json!({"kind":"move","destinationMs":10000}),
        json!({"kind":"remove"}),
    ] {
        assert!(group(&mut doc, &ids, action).unwrap_err().contains("锁定"));
        assert_eq!(doc, before);
    }
    group(&mut doc, &ids, json!({"kind":"copy","destinationMs":16500})).unwrap();
    let copied: Vec<_> = clips(&doc)
        .into_iter()
        .filter(|c| !old.iter().any(|s| s.id == c.id))
        .collect();
    assert_eq!(copied.len(), 2);
    assert_eq!(copied[1].end_ms, 20000);
    for (c, s) in copied.iter().zip(&old) {
        assert!(!c.locked);
        assert_eq!(c.scene_id, s.scene_id);
        assert_eq!(c.fade_ms, s.fade_ms);
        assert_eq!(c.end_ms - c.start_ms, s.end_ms - s.start_ms);
    }
    group(
        &mut doc,
        &copied.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        json!({"kind":"remove"}),
    )
    .unwrap();
    assert_eq!(doc, before);
}
#[test]
fn invalid_selection_collision_second_interval_overflow_and_capacity_are_atomic() {
    let mut doc = fixture();
    let old = clips(&doc);
    let before = doc.clone();
    let pair = vec![old[0].id.clone(), old[1].id.clone()];
    for ids in [
        vec![],
        vec![old[0].id.clone(); 2],
        vec!["missing".into()],
        vec![old[0].id.clone(); 513],
    ] {
        assert!(group(&mut doc, &ids, json!({"kind":"remove"})).is_err());
        assert_eq!(doc, before);
    }
    for (kind, target) in [
        ("move", 5000_u64),
        ("copy", 2000),
        ("copy", 16501),
        ("move", u64::MAX),
    ] {
        assert!(group(&mut doc, &pair, json!({"kind":kind,"destinationMs":target})).is_err());
        assert_eq!(doc, before);
    }
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    root["media"]["audioEditing"]["lightingClips"]=json!((0..512).map(|i|json!({"id":format!("a0000000-0000-4000-8000-{i:012}"),"name":"短段","sceneId":old[0].scene_id,"startMs":i*10,"endMs":i*10+5,"fadeMs":0,"locked":false})).collect::<Vec<_>>());
    let mut full = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let baseline = full.clone();
    let ids: Vec<_> = clips(&full).iter().map(|c| c.id.clone()).collect();
    assert!(
        group(
            &mut full,
            &ids[..1],
            json!({"kind":"copy","destinationMs":10000})
        )
        .unwrap_err()
        .contains("512")
    );
    assert_eq!(full, baseline);
    group(&mut full, &ids, json!({"kind":"remove"})).unwrap();
    assert!(clips(&full).is_empty());
}
#[test]
fn legacy_mode_and_unknown_fields_are_rejected() {
    let mut doc = fixture();
    let ids = vec![clips(&doc)[0].id.clone()];
    for action in [
        json!({"kind":"remove","destinationMs":1}),
        json!({"kind":"move","destinationMs":1,"extra":true}),
        json!({"kind":"copy","destinationMs":-1}),
    ] {
        assert!(serde_json::from_value::<EditCommand>(json!({"op":"audio","command":{"kind":"editLightingClips","ids":ids,"action":action}})).is_err());
    }
    edit(&mut doc, json!({"kind":"clear"})).unwrap();
    edit(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":20000}})).unwrap();
    let before = doc.clone();
    assert!(
        group(&mut doc, &ids, json!({"kind":"remove"}))
            .unwrap_err()
            .contains("转换")
    );
    assert_eq!(doc, before);
}

#[test]
fn group_fade_preserves_source_progress_and_all_other_clip_fields() {
    let mut doc = fixture();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    root["requires"].as_array_mut().unwrap().extend([
        json!({"key":"media.audio-clip-offset","version":1}),
        json!({"key":"media.audio-clip-state","version":1}),
    ]);
    root["media"]["audioEditing"]["lightingClips"][1]["effectOffsetMs"] = json!(725);
    root["media"]["audioEditing"]["lightingClips"][1]["enabled"] = json!(false);
    doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let before = clips(&doc);
    let ids = vec![before[2].id.clone(), before[1].id.clone()];
    for fade_ms in [1000, 0, 333] {
        group(&mut doc, &ids, json!({"kind":"fade","fadeMs":fade_ms})).unwrap();
        let mut expected = before.clone();
        expected[1].fade_ms = fade_ms;
        expected[2].fade_ms = fade_ms;
        assert_eq!(clips(&doc), expected);
        assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    }
}

#[test]
fn later_short_or_locked_clip_rejects_whole_fade_and_contract_is_strict() {
    let mut doc = fixture();
    let old = clips(&doc);
    let ids = vec![old[1].id.clone(), old[2].id.clone()];
    let before = doc.clone();
    for fade_ms in [1001_u64, u64::MAX] {
        let error = group(&mut doc, &ids, json!({"kind":"fade","fadeMs":fade_ms})).unwrap_err();
        assert!(error.contains(&old[2].name) || error.contains(&old[1].name));
        assert_eq!(doc, before);
    }
    edit(
        &mut doc,
        json!({"kind":"setLightingClipLock","id":old[2].id,"locked":true}),
    )
    .unwrap();
    let locked = doc.clone();
    assert!(
        group(&mut doc, &ids, json!({"kind":"fade","fadeMs":0}))
            .unwrap_err()
            .contains(&old[2].name)
    );
    assert_eq!(doc, locked);
    for action in [
        json!({"kind":"fade","fadeMs":-1}),
        json!({"kind":"fade","fadeMs":0.5}),
        json!({"kind":"fade","fadeMs":0,"enabled":true}),
        json!({"kind":"fade"}),
    ] {
        assert!(serde_json::from_value::<EditCommand>(json!({"op":"audio","command":{"kind":"editLightingClips","ids":ids,"action":action}})).is_err());
    }
}
