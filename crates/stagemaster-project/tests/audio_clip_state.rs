use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::Document;
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value(command).unwrap())
}
fn audio(doc: &mut Document, command: Value) -> Result<(), String> {
    let mut op = json!({"op":"audio"});
    op["command"] = command;
    edit(doc, op)
}
fn fixture() -> Document {
    let mut doc = Document::new("片段启停").unwrap();
    let v = doc.view();
    edit(&mut doc,json!({"op":"addFixture","name":"灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    for name in ["亮", "暗"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let v = doc.view();
    let a = &v.scenes[0].id;
    let b = &v.scenes[1].id;
    edit(&mut doc,json!({"op":"setSceneValue","sceneId":a,"fixtureId":v.fixtures[0].id,"attribute":"dimmer","mode":"literal","value":60000})).unwrap();
    audio(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":10000}})).unwrap();
    audio(&mut doc, json!({"kind":"convertLightingClips"})).unwrap();
    for (start, end, scene) in [(0, 2000, a), (2000, 4000, b), (4000, 6000, a)] {
        audio(&mut doc,json!({"kind":"addLightingClip","name":"段","sceneId":scene,"startMs":start,"endMs":end,"fadeMs":1000})).unwrap();
    }
    doc
}
fn set(doc: &mut Document, ids: &[String], enabled: bool) -> Result<(), String> {
    audio(
        doc,
        json!({"kind":"editLightingClips","ids":ids,"action":{"kind":"enabled","enabled":enabled}}),
    )
}
fn ids(doc: &Document) -> Vec<String> {
    doc.audio_timeline()
        .unwrap()
        .lighting_clips
        .unwrap()
        .iter()
        .map(|c| c.id.clone())
        .collect()
}
fn sample(doc: &Document, time: u64) -> Vec<u16> {
    let t = doc.audio_timeline().unwrap();
    let active = t.lighting_at(time);
    let c = doc.compile_audio_lighting(active.map(|r| r.id)).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    p.advance(time - active.map_or(0, |r| r.start_ms)).unwrap();
    p.values().to_vec()
}
#[test]
fn disabled_intervals_are_defaults_and_do_not_supply_neighbor_fades() {
    let mut doc = fixture();
    let before = doc.clone();
    let ids = ids(&doc);
    assert_eq!(sample(&doc, 2500), vec![30000]);
    set(&mut doc, &ids[..1], false).unwrap();
    assert!(doc.audio_timeline().unwrap().lighting_at(1999).is_none());
    assert!(
        doc.compile_audio_lighting(Some(&ids[0]))
            .err()
            .unwrap()
            .contains("停用")
    );
    for t in [0, 500, 1000, 1999, 2000, 2500, 3999, 1000] {
        assert_eq!(sample(&doc, t), vec![0], "at {t}");
    }
    assert_eq!(sample(&doc, 4500), vec![30000]);
    set(&mut doc, &ids[..1], true).unwrap();
    for t in (0..10000)
        .step_by(37)
        .chain([1999, 2000, 2500, 3999, 4000, 5999, 6000])
    {
        assert_eq!(sample(&doc, t), sample(&before, t));
    }
}
#[test]
fn group_state_is_atomic_lock_safe_and_copy_preserves_disabled_source() {
    let mut doc = fixture();
    let ids = ids(&doc);
    audio(
        &mut doc,
        json!({"kind":"setLightingClipLock","id":ids[1],"locked":true}),
    )
    .unwrap();
    let before = doc.clone();
    assert!(
        set(&mut doc, &ids[..2], false)
            .unwrap_err()
            .contains("锁定")
    );
    assert_eq!(doc, before);
    set(&mut doc, &ids[..1], false).unwrap();
    let disabled = doc.clone();
    audio(
        &mut doc,
        json!({"kind":"copyLightingClip","id":ids[0],"startMs":6000}),
    )
    .unwrap();
    let clips = doc.audio_timeline().unwrap().lighting_clips.unwrap();
    assert!(!clips[3].enabled && !clips[3].locked);
    assert_eq!(sample(&doc, 6500), vec![0]);
    // A disabled interval still owns its place and keeps its scene reference.
    let mut c = serde_json::to_value(&clips[0]).unwrap();
    c["endMs"] = json!(2500);
    assert!(audio(&mut doc, json!({"kind":"putLightingClip","clip":c})).is_err());
    let mut c = serde_json::to_value(&clips[0]).unwrap();
    c["enabled"] = json!(true);
    assert!(
        audio(&mut doc, json!({"kind":"putLightingClip","clip":c}))
            .unwrap_err()
            .contains("恢复")
    );
    let mut d = disabled.clone();
    audio(&mut d, json!({"kind":"removeLightingClip","id":ids[2]})).unwrap();
    let referenced = d.clone();
    assert!(edit(&mut d, json!({"op":"removeScene","id":clips[0].scene_id})).is_err());
    assert_eq!(d, referenced);
}
#[test]
fn compatible_defaults_capability_and_schema_are_strict() {
    let mut doc = fixture();
    let ids = ids(&doc);
    let encoded = doc.encode().unwrap();
    assert!(
        !String::from_utf8(encoded.clone())
            .unwrap()
            .contains("\"enabled\"")
    );
    assert_eq!(doc, Document::decode(&encoded).unwrap());
    set(&mut doc, &ids[..1], false).unwrap();
    let root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert!(
        root["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "media.audio-clip-state")
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    for case in 0..5 {
        let mut r = root.clone();
        match case {
            0 => r["requires"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v["key"] != "media.audio-clip-state"),
            1 => r["media"]["audioEditing"]["lightingClips"][0]["enabled"] = json!("false"),
            2 => r["media"]["audioEditing"]["lightingClips"][0]["enabled"] = Value::Null,
            3 => {
                r.as_object_mut().unwrap().remove("media");
            }
            _ => {
                r["media"]["audioEditing"]
                    .as_object_mut()
                    .unwrap()
                    .remove("lightingClips");
            }
        }
        assert!(
            Document::decode(&serde_json::to_vec(&r).unwrap()).is_err(),
            "case {case}"
        );
    }
    for action in [
        json!({"kind":"enabled","enabled":true,"unknown":1}),
        json!({"kind":"enabled","enabled":"true"}),
    ] {
        assert!(serde_json::from_value::<stagemaster_project::EditCommand>(json!({"op":"audio","command":{"kind":"editLightingClips","ids":ids,"action":action}})).is_err());
    }
    audio(&mut doc, json!({"kind":"clear"})).unwrap();
    assert!(
        !String::from_utf8(doc.encode().unwrap())
            .unwrap()
            .contains("media.audio-")
    );
}
