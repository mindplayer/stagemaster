use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand};

fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value(command).unwrap())
}
fn fixture() -> (Document, String, String, String) {
    let mut doc = Document::new("渐变验收").unwrap();
    let v = doc.view();
    edit(&mut doc, json!({"op":"addFixture","name":"亮度灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    let lamp = doc.view().fixtures[0].id.clone();
    for name in ["亮场", "暗场"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let a = doc.view().scenes[0].id.clone();
    let b = doc.view().scenes[1].id.clone();
    edit(&mut doc, json!({"op":"setSceneValue","sceneId":a,"fixtureId":lamp,"attribute":"dimmer","mode":"literal","value":60000})).unwrap();
    edit(&mut doc, json!({"op":"audio","command":{"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"音乐.wav","extension":"wav","durationMs":10000}}})).unwrap();
    (doc, lamp, a, b)
}
fn marker(index: u8, time: u64, scene: Option<&str>, fade: u64) -> Value {
    json!({"id":format!("a0000000-0000-4000-8000-{index:012}"),"name":format!("段落 {index}"),"timeMs":time,"sceneId":scene,"fadeMs":fade})
}
fn put(doc: &mut Document, value: Value) -> Result<(), String> {
    let mut command = json!({"op":"audio","command":{"kind":"putMarker"}});
    command["command"]["marker"] = value;
    edit(doc, command)
}
fn sample(doc: &Document, time: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let marker = track.scene_at(time);
    let compiled = doc
        .compile_audio_marker(marker.map(|m| m.id.as_str()))
        .unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    player
        .advance(time - marker.map_or(0, |m| m.time_ms))
        .unwrap();
    player.values().to_vec()
}
#[test]
fn defaults_first_entry_and_static_transition_are_identical_after_arbitrary_seek() {
    let (mut doc, _, a, b) = fixture();
    put(&mut doc, marker(1, 1000, Some(&a), 1000)).unwrap();
    put(&mut doc, marker(2, 4000, Some(&b), 2000)).unwrap();
    put(&mut doc, marker(3, 4500, None, 0)).unwrap();
    for (time, value) in [
        (999, 0),
        (1000, 0),
        (1500, 30000),
        (2000, 60000),
        (4000, 60000),
        (5000, 30000),
        (6000, 0),
        (1500, 30000),
        (9999, 0),
    ] {
        assert_eq!(sample(&doc, time), vec![value], "at {time}");
    }
    let compiled = doc
        .compile_audio_marker(Some("a0000000-0000-4000-8000-000000000002"))
        .unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    for time in (4000..6000).step_by(17) {
        player.advance(time - 4000).unwrap();
        assert_eq!(player.values(), sample(&doc, time));
    }
}
fn effect(doc: &mut Document, scene: &str, lamp: &str, id: &str) {
    edit(doc,json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{"id":id,"name":"上升","enabled":true,"fixtureIds":[lamp],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,"channels":[{"attribute":"dimmer","low":0,"high":60000}]}}})).unwrap();
}
#[test]
fn outgoing_dynamic_freezes_at_boundary_and_incoming_effect_uses_its_own_phase() {
    let (mut doc, lamp, a, b) = fixture();
    effect(&mut doc, &a, &lamp, "b0000000-0000-4000-8000-000000000001");
    effect(&mut doc, &b, &lamp, "b0000000-0000-4000-8000-000000000002");
    put(&mut doc, marker(1, 1000, Some(&a), 300)).unwrap();
    put(&mut doc, marker(2, 2250, Some(&b), 1000)).unwrap();
    let source = doc.compile_scene(&a).unwrap();
    let mut old = Player::new(source.plan, 0);
    old.execute(0, 0).unwrap();
    old.advance(1250).unwrap();
    let frozen = u64::from(old.values()[0]);
    let target = doc.compile_scene(&b).unwrap();
    let mut next = Player::new(target.plan, 0);
    next.execute(0, 0).unwrap();
    next.advance(500).unwrap();
    let expected = u16::try_from((frozen + u64::from(next.values()[0])).div_ceil(2)).unwrap();
    assert_eq!(sample(&doc, 2250)[0], u16::try_from(frozen).unwrap());
    assert_eq!(sample(&doc, 2750)[0], expected);
    assert_ne!(sample(&doc, 3250), sample(&doc, 2750));
}
#[test]
fn invalid_neighbor_changes_and_trim_are_atomic_and_capability_is_required() {
    let (mut doc, _, a, b) = fixture();
    put(&mut doc, marker(1, 1000, Some(&a), 2000)).unwrap();
    put(&mut doc, marker(2, 4000, Some(&b), 1000)).unwrap();
    let before = doc.clone();
    for command in [
        json!({"kind":"putMarker","marker":marker(2,2000,Some(&b),0)}),
        json!({"kind":"putMarker","marker":marker(3,500,None,10)}),
        json!({"kind":"trim","inMs":0,"outMs":4500}),
        json!({"kind":"putMarker","marker":marker(1,1000,Some(&a),3001)}),
    ] {
        assert!(edit(&mut doc, json!({"op":"audio","command":command})).is_err());
        assert_eq!(doc, before);
    }
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "media.audio-transitions");
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap())
            .unwrap_err()
            .contains("渐变能力")
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    edit(&mut doc, json!({"op":"audio","command":{"kind":"clear"}})).unwrap();
    let cleared = String::from_utf8(doc.encode().unwrap()).unwrap();
    assert!(!cleared.contains("media.audio-"));
}
#[test]
fn legacy_marker_has_no_new_fields_or_capability_and_retains_hard_cut_output() {
    let (mut doc, _, a, _) = fixture();
    let mut m = marker(1, 1000, Some(&a), 0);
    m.as_object_mut().unwrap().remove("fadeMs");
    put(&mut doc, m).unwrap();
    assert_eq!(sample(&doc, 999), vec![0]);
    assert_eq!(sample(&doc, 1000), vec![60000]);
    let bytes = doc.encode().unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains("fadeMs"));
    assert!(!text.contains("media.audio-transitions"));
    assert_eq!(doc, Document::decode(&bytes).unwrap());
    assert!(doc.compile_audio_marker(Some("missing")).is_err());
}
#[test]
fn function_channels_switch_at_start_while_brightness_fades() {
    let mut doc = Document::new("功能渐变").unwrap();
    edit(&mut doc,json!({"op":"fixture","command":{"op":"saveProfile","definition":{
        "name":"色盘灯","manufacturer":"测试","model":"两通道","mode":"两通道","footprint":2,
        "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":0},
        {"attribute":"color-wheel","coarse":2,"fine":null,"defaultValue":{"functionKey":"white","position":0},
        "functions":[{"key":"white","name":"白色","mode":"slot","dmxFrom":0,"dmxTo":15,"dmxDefault":0},
        {"key":"red","name":"红色","mode":"slot","dmxFrom":16,"dmxTo":31,"dmxDefault":20}]}]}}})).unwrap();
    let v = doc.view();
    edit(&mut doc,json!({"op":"addFixture","name":"灯","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    edit(&mut doc, json!({"op":"addScene","name":"入场"})).unwrap();
    let v = doc.view();
    let scene = v.scenes[0].id.clone();
    let fixture = v.fixtures[0].id.clone();
    edit(&mut doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":"dimmer","mode":"literal","value":60000})).unwrap();
    doc.edit(serde_json::from_value::<EditCommand>(json!({"op":"audio","command":{"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"音乐.wav","extension":"wav","durationMs":10000}}})).unwrap()).unwrap();
    edit(&mut doc,json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":fixture,"attribute":"color-wheel","selection":{"functionKey":"red","position":0}})).unwrap();
    put(&mut doc, marker(1, 1000, Some(&scene), 1000)).unwrap();
    let compiled = doc
        .compile_audio_marker(Some("a0000000-0000-4000-8000-000000000001"))
        .unwrap();
    let snaps = compiled.plan.snap_attributes().to_vec();
    let target = compiled.plan.steps()[0].target.clone();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    assert_eq!(player.values()[0], 0);
    player.advance(500).unwrap();
    assert_eq!(player.values()[0], 30000);
    for index in snaps {
        assert_eq!(
            player.values()[usize::from(index)],
            target[usize::from(index)]
        );
    }
}
