use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::Document;
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value(command).unwrap())
}
fn audio(doc: &mut Document, command: Value) -> Result<(), String> {
    let mut wrapped = json!({"op":"audio"});
    wrapped["command"] = command;
    edit(doc, wrapped)
}
fn fixture() -> (Document, String, String) {
    let mut doc = Document::new("独立灯光片段").unwrap();
    let v = doc.view();
    edit(&mut doc, json!({"op":"addFixture","name":"测试灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    for name in ["亮场", "暗场"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let v = doc.view();
    let a = v.scenes[0].id.clone();
    let b = v.scenes[1].id.clone();
    edit(&mut doc,json!({"op":"setSceneValue","sceneId":a,"fixtureId":v.fixtures[0].id,"attribute":"dimmer","mode":"literal","value":60000})).unwrap();
    audio(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"音乐.wav","extension":"wav","durationMs":10000}})).unwrap();
    (doc, a, b)
}
fn add(doc: &mut Document, scene: &str, start: u64, end: u64, fade: u64) -> Result<(), String> {
    audio(
        doc,
        json!({"kind":"addLightingClip","name":"测试片段","sceneId":scene,"startMs":start,"endMs":end,"fadeMs":fade}),
    )
}
fn sample(doc: &Document, time: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let active = track.lighting_at(time);
    let c = doc.compile_audio_lighting(active.map(|r| r.id)).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    p.advance(time - active.map_or(0, |r| r.start_ms)).unwrap();
    p.values().to_vec()
}
#[test]
fn explicit_conversion_preserves_beats_and_boundary_and_interior_samples_without_auto_migration() {
    let (mut doc, a, b) = fixture();
    for (i, time, scene, fade) in [
        (1, 1000, Some(&a), 1000),
        (2, 3000, None, 0),
        (3, 4000, Some(&b), 1000),
    ] {
        audio(&mut doc,json!({"kind":"putMarker","marker":{"id":format!("a0000000-0000-4000-8000-{i:012}"),"name":"节拍","timeMs":time,"sceneId":scene,"fadeMs":fade}})).unwrap();
    }
    let old = Document::decode(&doc.encode().unwrap()).unwrap();
    assert!(old.audio_timeline().unwrap().lighting_clips.is_none());
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    let track = doc.audio_timeline().unwrap();
    let clips = track.lighting_clips.unwrap();
    assert_eq!(clips.len(), 2);
    assert_eq!((clips[0].start_ms, clips[0].end_ms), (1000, 4000));
    for (before, after) in old
        .audio_timeline()
        .unwrap()
        .markers
        .iter()
        .zip(&track.markers)
    {
        assert_eq!(
            (&before.id, &before.name, before.time_ms),
            (&after.id, &after.name, after.time_ms)
        );
        assert!(after.scene_id.is_none());
        assert_eq!(after.fade_ms, 0);
    }
    // Compare boundaries, interior samples and backward jumps.
    for time in (0..10000)
        .step_by(37)
        .chain([999, 1000, 1999, 2000, 3999, 4000, 4500, 9999, 1500])
    {
        assert_eq!(sample(&old, time), sample(&doc, time), "at {time}");
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    assert!(audio(&mut doc, json!({"kind":"convertLightingClips"})).is_err());
}
#[test]
fn gaps_release_defaults_and_only_adjacent_clips_supply_entry_snapshot() {
    let (mut doc, a, b) = fixture();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    add(&mut doc, &a, 1000, 3000, 1000).unwrap();
    add(&mut doc, &b, 3000, 5000, 1000).unwrap();
    add(&mut doc, &a, 6000, 10000, 1000).unwrap();
    for (time, level) in [
        (999, 0),
        (1500, 30000),
        (2999, 60000),
        (3000, 60000),
        (3500, 30000),
        (5000, 0),
        (6000, 0),
        (6500, 30000),
        (10000, 0),
        (2500, 60000),
    ] {
        assert_eq!(sample(&doc, time), vec![level], "at {time}");
    }
    let id = doc.audio_timeline().unwrap().lighting_clips.unwrap()[1]
        .id
        .clone();
    let c = doc.compile_audio_lighting(Some(&id)).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    for time in (3000..5000).step_by(17) {
        p.advance(time - 3000).unwrap();
        assert_eq!(p.values(), sample(&doc, time));
    }
}
#[test]
fn invalid_edits_are_atomic_and_never_push_neighbors_or_beats() {
    let (mut doc, a, _) = fixture();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    add(&mut doc, &a, 1000, 3000, 1000).unwrap();
    add(&mut doc, &a, 4000, 6000, 0).unwrap();
    let baseline = doc.clone();
    let clip =
        serde_json::to_value(&doc.audio_timeline().unwrap().lighting_clips.unwrap()[0]).unwrap();
    for (key, value) in [
        ("startMs", json!(2001)),
        ("endMs", json!(4500)),
        ("fadeMs", json!(2001)),
        ("sceneId", json!("a0000000-0000-4000-8000-000000000001")),
        ("name", json!(" ")),
    ] {
        let mut c = clip.clone();
        c[key] = value;
        assert!(audio(&mut doc, json!({"kind":"putLightingClip","clip":c})).is_err());
        assert_eq!(doc, baseline);
    }
    for cmd in [
        json!({"kind":"trim","inMs":0,"outMs":5999}),
        json!({"kind":"copyLightingClip","id":clip["id"],"startMs":5000}),
        json!({"kind":"copyLightingClip","id":clip["id"],"startMs":u64::MAX}),
    ] {
        assert!(audio(&mut doc, cmd).is_err());
        assert_eq!(doc, baseline);
    }
    assert!(add(&mut doc, &a, 2000, 4000, 0).is_err());
    assert_eq!(doc, baseline);
    let mut moved = clip;
    moved["startMs"] = json!(0);
    moved["endMs"] = json!(2000);
    audio(&mut doc, json!({"kind":"putLightingClip","clip":moved})).unwrap();
    assert_eq!(
        doc.audio_timeline().unwrap().lighting_clips.unwrap()[1],
        baseline.audio_timeline().unwrap().lighting_clips.unwrap()[1]
    );
}
#[test]
fn locked_clip_rejects_mutation_and_delete_but_copy_has_independent_unlocked_identity() {
    let (mut doc, a, _) = fixture();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    add(&mut doc, &a, 1000, 3000, 200).unwrap();
    let id = doc.audio_timeline().unwrap().lighting_clips.unwrap()[0]
        .id
        .clone();
    audio(
        &mut doc,
        json!({"kind":"setLightingClipLock","id":id,"locked":true}),
    )
    .unwrap();
    let before = doc.clone();
    let mut c =
        serde_json::to_value(&doc.audio_timeline().unwrap().lighting_clips.unwrap()[0]).unwrap();
    c["locked"] = json!(false);
    for cmd in [
        json!({"kind":"removeLightingClip","id":id}),
        json!({"kind":"putLightingClip","clip":c}),
    ] {
        assert!(audio(&mut doc, cmd).is_err());
        assert_eq!(doc, before);
    }
    audio(
        &mut doc,
        json!({"kind":"copyLightingClip","id":id,"startMs":5000}),
    )
    .unwrap();
    let clips = doc.audio_timeline().unwrap().lighting_clips.unwrap();
    assert!(clips[0].locked);
    assert!(!clips[1].locked);
    assert_ne!(clips[0].id, clips[1].id);
    assert_eq!(
        (clips[1].start_ms, clips[1].end_ms, clips[1].fade_ms),
        (5000, 7000, 200)
    );
    audio(
        &mut doc,
        json!({"kind":"removeLightingClip","id":clips[1].id}),
    )
    .unwrap();
    assert_eq!(doc, before);
}
#[test]
fn format_capability_identity_reference_and_dual_scheduling_are_rejected() {
    let (mut doc, a, _) = fixture();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    add(&mut doc, &a, 0, 10000, 0).unwrap();
    let before = doc.clone();
    assert!(edit(&mut doc, json!({"op":"removeScene","id":a})).is_err());
    assert_eq!(doc, before);
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for case in 0..6 {
        let mut r = root.clone();
        match case {
            0 => r["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v["key"] != "media.audio-clips"),
            1 => {
                r["media"]["audioEditing"]
                    .as_object_mut()
                    .unwrap()
                    .remove("lightingClips");
            }
            2 => r["media"]["audioEditing"]["lightingClips"][0]["id"] = json!(a),
            3 => r["media"]["audioEditing"]["lightingClips"][0]["locked"] = json!("yes"),
            4 => {
                r["media"]["audioEditing"]["markers"] = json!([{"id":"a0000000-0000-4000-8000-000000000001","name":"节拍","timeMs":0,"sceneId":a}]);
            }
            _ => r["media"]["audioEditing"]["lightingClips"][0]["endMs"] = json!(10001),
        }
        assert!(
            Document::decode(&serde_json::to_vec(&r).unwrap()).is_err(),
            "case {case}"
        );
    }
    audio(&mut doc, json!({"kind":"clear"})).unwrap();
    assert!(
        !String::from_utf8(doc.encode().unwrap())
            .unwrap()
            .contains("media.audio-")
    );
}
#[test]
fn capacity_and_unknown_fields_are_rejected_before_mutation() {
    let (mut doc, a, _) = fixture();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    root["media"]["audioEditing"]["lightingClips"]=Value::Array((0..512).map(|i|json!({"id":format!("b0000000-0000-4000-8000-{i:012}"),"name":"短段","sceneId":a,"startMs":i*10,"endMs":i*10+5,"fadeMs":0,"locked":false})).collect());
    let mut full = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let before = full.clone();
    assert!(add(&mut full, &a, 9000, 10000, 0).is_err());
    assert_eq!(full, before);
    assert!(audio(&mut full,json!({"kind":"copyLightingClip","id":"b0000000-0000-4000-8000-000000000000","startMs":9000})).is_err());
    assert_eq!(full, before);
    root["media"]["audioEditing"]["lightingClips"][0]["unknown"] = json!(0);
    assert!(Document::decode(&serde_json::to_vec(&root).unwrap()).is_err());
}
#[test]
fn dynamic_conversion_and_copied_phase_remain_deterministic() {
    let (mut doc, a, b) = fixture();
    let fixture = doc.view().fixtures[0].id.clone();
    edit(&mut doc,json!({"op":"effect","command":{"kind":"put","sceneId":a,"effect":{"id":"b0000000-0000-4000-8000-000000000010","name":"波动","enabled":true,"fixtureIds":[fixture],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,"channels":[{"attribute":"dimmer","low":0,"high":60000}]}}})).unwrap();
    for (i, time, scene, fade) in [(1, 1000, &a, 200), (2, 2250, &b, 1000)] {
        audio(&mut doc,json!({"kind":"putMarker","marker":{"id":format!("c0000000-0000-4000-8000-{i:012}"),"name":"动态","timeMs":time,"sceneId":scene,"fadeMs":fade}})).unwrap();
    }
    let old = doc.clone();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    for time in [999, 1000, 1100, 1750, 2249, 2250, 2750, 3250, 1250] {
        assert_eq!(sample(&doc, time), sample(&old, time), "at {time}");
    }
    let clips = doc.audio_timeline().unwrap().lighting_clips.unwrap();
    audio(
        &mut doc,
        json!({"kind":"removeLightingClip","id":clips[1].id}),
    )
    .unwrap();
    audio(
        &mut doc,
        json!({"kind":"copyLightingClip","id":clips[0].id,"startMs":6000}),
    )
    .unwrap();
    for offset in [0, 100, 200, 250, 500, 1000, 1249] {
        assert_eq!(sample(&doc, 1000 + offset), sample(&doc, 6000 + offset));
    }
}
