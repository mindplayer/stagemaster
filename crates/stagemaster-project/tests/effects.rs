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
