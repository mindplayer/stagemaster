use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::Document;

fn setup() -> (Document, String, Vec<String>) {
    let mut doc = Document::new("效果测试").unwrap();
    let p = doc.view();
    for i in 0..4 {
        edit(&mut doc,json!({"op":"addFixture","name":format!("灯 {}",i+1),"profileId":p.profiles[1].id,"domainId":p.domains[0].id,"universe":1,"address":1+i*4})).unwrap();
    }
    edit(&mut doc, json!({"op":"addScene","name":"追逐"})).unwrap();
    let p = doc.view();
    (
        doc,
        p.scenes[0].id.clone(),
        p.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value(command).unwrap())
}
fn effect(ids: &[String]) -> Value {
    json!({"id":"29999999-0000-4000-8000-000000000001","name":"追逐","enabled":true,"fixtureIds":ids,"periodMs":1000,"spreadDegrees":360,"phaseDegrees":0,"reverse":false,"waveform":"pulse","dutyPercent":25,"channels":[{"attribute":"dimmer","low":0,"high":65535}]})
}
fn put(doc: &mut Document, scene: &str, e: &Value) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":e}}),
    )
}

#[test]
fn ordered_effects_survive_save_compile_and_encode_real_dmx_slots() {
    let (mut doc, scene, mut ids) = setup();
    ids.swap(0, 2);
    put(&mut doc, &scene, &effect(&ids)).unwrap();
    let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
    assert_eq!(reopened.view().scenes[0].effects[0].fixture_ids, ids);
    let compiled = reopened.compile_scene(&scene).unwrap();
    let mut p = Player::new(compiled.plan, 0);
    p.execute(0, 0).unwrap();
    for (time, address) in [(0, 9), (250, 5), (500, 1), (750, 13)] {
        p.advance(time).unwrap();
        let out = compiled.output.render(p.values()).unwrap();
        assert_eq!(out.slots[address - 1], 255);
        assert_eq!(out.slots.iter().filter(|v| **v > 0).count(), 1);
    }
}
#[test]
fn reverse_uses_saved_order_and_disabled_effect_restores_static_values() {
    let (mut doc, scene, ids) = setup();
    let mut e = effect(&ids);
    e["reverse"] = json!(true);
    put(&mut doc, &scene, &e).unwrap();
    let c = doc.compile_scene(&scene).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    assert_eq!(c.output.render(p.values()).unwrap().slots[12], 255);
    e["enabled"] = json!(false);
    put(&mut doc, &scene, &e).unwrap();
    let c = doc.compile_scene(&scene).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    assert!(p.values().iter().all(|v| *v == 0));
}
#[test]
fn incompatible_conflicting_and_dangling_effects_are_atomic_errors() {
    let (mut doc, scene, ids) = setup();
    let e = effect(&ids);
    put(&mut doc, &scene, &e).unwrap();
    let before = doc.clone();
    let mut duplicate = e.clone();
    duplicate["id"] = json!("29999999-0000-4000-8000-000000000002");
    assert!(
        put(&mut doc, &scene, &duplicate)
            .unwrap_err()
            .contains("同一灯具属性")
    );
    assert_eq!(doc, before);
    for bad in [json!(0), json!(99), json!(3_600_001)] {
        let mut invalid = e.clone();
        invalid["periodMs"] = bad;
        assert!(put(&mut doc, &scene, &invalid).is_err());
        assert_eq!(doc, before);
    }
    let mut invalid = e.clone();
    invalid["fixtureIds"] = json!([ids[0], ids[0]]);
    assert!(put(&mut doc, &scene, &invalid).is_err());
    assert_eq!(doc, before);
    let mut invalid = e.clone();
    invalid["channels"] = json!([{"attribute":"pan","low":0,"high":100}]);
    assert!(put(&mut doc, &scene, &invalid).is_err());
    assert_eq!(doc, before);
    // Remove every static assignment first, so the effect reference alone protects the fixture.
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["lighting"]["scenes"][0]["assignments"]
        .as_array_mut()
        .unwrap()
        .retain(|a| a["target"]["fixtureId"] != ids[0]);
    let mut only_effect = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(
        edit(&mut only_effect, json!({"op":"removeFixture","id":ids[0]}))
            .unwrap_err()
            .contains("效果")
    );
    duplicate["enabled"] = json!(false);
    put(&mut doc, &scene, &duplicate).unwrap();
}
#[test]
fn copying_scenes_renews_nested_ids_and_requires_guard_prevents_silent_static_open() {
    let (mut doc, scene, ids) = setup();
    put(&mut doc, &scene, &effect(&ids)).unwrap();
    edit(
        &mut doc,
        json!({"op":"duplicateScene","id":scene,"name":"副本"}),
    )
    .unwrap();
    let p = doc.view();
    assert_ne!(p.scenes[0].effects[0].id, p.scenes[1].effects[0].id);
    assert_eq!(
        p.scenes[0].effects[0].fixture_ids,
        p.scenes[1].effects[0].fixture_ids
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "lighting.effects.basic");
    assert!(
        Document::decode(&serde_json::to_vec(&raw).unwrap())
            .unwrap_err()
            .contains("能力声明")
    );
}
#[test]
fn dimmer_and_rgb_can_combine_and_scene_and_list_compile_identically() {
    let (mut doc, scene, ids) = setup();
    put(&mut doc, &scene, &effect(&ids)).unwrap();
    let mut color = effect(&ids);
    color["id"] = json!("29999999-0000-4000-8000-000000000002");
    color["waveform"] = json!("triangle");
    color["spreadDegrees"] = json!(0);
    color["channels"] =
        json!([{"attribute":"red","low":0,"high":65535},{"attribute":"blue","low":65535,"high":0}]);
    put(&mut doc, &scene, &color).unwrap();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"测试列表","sceneId":scene}}),
    )
    .unwrap();
    let v = doc.view();
    let seq = &v.sequences[0];
    let s = &seq.steps[0];
    edit(&mut doc,json!({"op":"sequence","command":{"kind":"updateStep","id":seq.id,"stepId":s.id,"name":s.name,"number":s.number,"sceneId":scene,"delayMs":0,"fadeMs":0,"waitMs":null}})).unwrap();
    let mut a = Player::new(doc.compile_scene(&scene).unwrap().plan, 0);
    let mut b = Player::new(doc.compile_sequence(&seq.id).unwrap().plan, 0);
    a.execute(0, 0).unwrap();
    b.execute(0, 0).unwrap();
    for time in [0, 125, 500, 999, 1000, 900_001] {
        a.advance(time).unwrap();
        b.advance(time).unwrap();
        assert_eq!(a.values(), b.values());
    }
}

fn keyed_effect(ids: &[String]) -> Value {
    let mut e = effect(ids);
    e["waveform"] = json!("keyframes");
    e["spreadDegrees"] = json!(0);
    e["channels"] = json!([{"attribute":"red","keyframes":[{"position":0,"value":0,"transition":"linear"},{"position":2500,"value":65535,"transition":"hold"},{"position":7500,"value":10000,"transition":"smooth"}]}]);
    e
}
#[test]
fn keyframes_save_copy_and_compile_as_independent_curve_data() {
    let (mut doc, scene, ids) = setup();
    let e = keyed_effect(&ids);
    put(&mut doc, &scene, &e).unwrap();
    let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
    let raw: Value = serde_json::from_slice(&reopened.encode().unwrap()).unwrap();
    assert!(
        raw["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "lighting.effects.keyframes")
    );
    let compiled = reopened.compile_scene(&scene).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    for (now, value) in [(0, 0), (250, 255), (500, 255), (750, 39), (1000, 0)] {
        player.advance(now).unwrap();
        assert_eq!(
            compiled.output.render(player.values()).unwrap().slots[1],
            value
        );
    }
    edit(
        &mut doc,
        json!({"op":"duplicateScene","id":scene,"name":"独立副本"}),
    )
    .unwrap();
    let other = doc.view().scenes[1].id.clone();
    let mut changed = e.clone();
    changed["channels"][0]["keyframes"][1]["value"] = json!(0);
    put(&mut doc, &scene, &changed).unwrap();
    let copied = doc.compile_scene(&other).unwrap();
    let mut player = Player::new(copied.plan, 0);
    player.execute(0, 0).unwrap();
    player.advance(250).unwrap();
    assert_eq!(copied.output.render(player.values()).unwrap().slots[1], 255);
}
#[test]
fn malformed_timing_mode_and_missing_keyframe_capability_fail_without_mutating_document() {
    let (mut doc, scene, ids) = setup();
    let original = keyed_effect(&ids);
    put(&mut doc, &scene, &original).unwrap();
    let before = doc.clone();
    for positions in [[1, 2500, 7500], [0, 7500, 2500], [0, 2500, 2500]] {
        let mut e = original.clone();
        for (frame, pos) in e["channels"][0]["keyframes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(positions)
        {
            frame["position"] = json!(pos);
        }
        assert!(put(&mut doc, &scene, &e).is_err());
        assert_eq!(doc, before);
    }
    let mut e = original.clone();
    e["waveform"] = json!("smooth");
    assert!(put(&mut doc, &scene, &e).is_err());
    assert_eq!(doc, before);
    let mut e = original.clone();
    let mut second = e["channels"][0].clone();
    second["attribute"] = json!("blue");
    second["keyframes"][1]["transition"] = json!("smooth");
    e["channels"].as_array_mut().unwrap().push(second);
    assert!(put(&mut doc, &scene, &e).unwrap_err().contains("一致"));
    assert_eq!(doc, before);
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "lighting.effects.keyframes");
    assert!(
        Document::decode(&serde_json::to_vec(&raw).unwrap())
            .unwrap_err()
            .contains("能力声明")
    );
}
#[test]
fn keyframe_wire_format_rejects_unknown_or_ambiguous_values_before_deserializing() {
    let (mut doc, scene, ids) = setup();
    put(&mut doc, &scene, &keyed_effect(&ids)).unwrap();
    let raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for broken in [
        json!([]),
        json!([{"position":0,"value":0,"transition":"sine"},{"position":5000,"value":1,"transition":"hold"}]),
        json!([{"position":0,"value":0,"transition":"hold"},{"position":10000,"value":1,"transition":"hold"}]),
    ] {
        let mut invalid = raw.clone();
        invalid["lighting"]["scenes"][0]["effects"][0]["channels"][0]["keyframes"] = broken;
        assert!(Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }
    let mut invalid = raw;
    invalid["lighting"]["scenes"][0]["effects"][0]["channels"][0]["low"] = json!(0);
    assert!(Document::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn converting_pulse_to_keyframes_preserves_quantized_phase_and_order() {
    let (mut doc, scene, ids) = setup();
    let mut original = effect(&ids);
    original["dutyPercent"] = json!(33);
    put(&mut doc, &scene, &original).unwrap();
    let mut before = Player::new(doc.compile_scene(&scene).unwrap().plan, 0);
    original["waveform"] = json!("keyframes");
    original["channels"] = json!([{"attribute":"dimmer","keyframes":[{"position":0,"value":65535,"transition":"hold"},{"position":3300,"value":0,"transition":"hold"}]}]);
    put(&mut doc, &scene, &original).unwrap();
    let mut after = Player::new(doc.compile_scene(&scene).unwrap().plan, 0);
    before.execute(0, 0).unwrap();
    after.execute(0, 0).unwrap();
    for now in 0..=2017 {
        before.advance(now).unwrap();
        after.advance(now).unwrap();
        assert_eq!(before.values(), after.values(), "at {now}");
    }
}
#[test]
fn compiler_rejects_oversized_keyframe_lists_before_building_the_full_plan() {
    let (mut doc, scene, ids) = setup();
    let mut e = keyed_effect(&ids);
    let frames: Vec<_> = (0..32)
        .map(|i| json!({"position":i*300,"value":i*2000,"transition":"linear"}))
        .collect();
    e["channels"] =
        json!([{"attribute":"red","keyframes":frames},{"attribute":"blue","keyframes":frames}]);
    put(&mut doc, &scene, &e).unwrap();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"容量测试","sceneId":scene}}),
    )
    .unwrap();
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let sequence = &mut raw["lighting"]["sequences"][0];
    let id = sequence["id"].as_str().unwrap().to_owned();
    let step = sequence["steps"][0].clone();
    sequence["steps"] = Value::Array(
        (0..513)
            .map(|i| {
                let mut s = step.clone();
                s["id"] = json!(format!("28888888-0000-4000-8000-{i:012}"));
                s["number"] = json!((i + 1).to_string());
                s
            })
            .collect(),
    );
    let doc = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert!(
        doc.compile_sequence(&id)
            .err()
            .unwrap()
            .contains("关键帧超出")
    );
}
