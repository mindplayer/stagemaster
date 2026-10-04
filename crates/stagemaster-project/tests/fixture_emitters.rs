use serde_json::{Value, json};
use stagemaster_playback::{OutputMaster, Player};
use stagemaster_project::{Document, ProfileFile};
#[path = "support/fixture_emitters.rs"]
mod support;
use support::{definition, edit, save, setup};
#[test]
fn independent_emitters_have_explicit_ownership_and_lossless_files() {
    let (doc, _, scene) = setup(definition());
    let view = doc.view();
    assert!(view.profiles.last().unwrap().authorable);
    assert_eq!(view.fixtures[0].attributes[1].label, "图案光源 · 亮度");
    assert_eq!(view.fixtures[0].attributes[5].label, "染色光源 · 白光");
    let compiled = doc.compile_scene(&scene).unwrap();
    let slots = compiled
        .output
        .render(compiled.plan.defaults())
        .unwrap()
        .slots;
    assert_eq!(
        &slots[16..34],
        &[0, 0, 0, 0, 0, 255, 128, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(&slots[34..52], &slots[16..34]);
    let bytes = doc.encode().unwrap();
    assert_eq!(Document::decode(&bytes).unwrap(), doc);
    let raw: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        raw["requires"]
            .as_array()
            .unwrap()
            .contains(&json!({"key":"lighting.fixture-emitters","version":1}))
    );
    let file = doc
        .profile_file(&view.profiles.last().unwrap().id)
        .unwrap()
        .encode()
        .unwrap();
    assert_eq!(
        serde_json::to_value(ProfileFile::decode(&file).unwrap().definition()).unwrap()["emitters"],
        definition()["emitters"]
    );
}
#[test]
fn global_dimmer_is_scaled_once_and_white_is_independent() {
    let (mut doc, ids, scene) = setup(definition());
    for (key, value) in [
        ("emitter.pattern.dimmer", 65535),
        ("emitter.wash.white", 65535),
    ] {
        edit(&mut doc, json!({"op":"setSceneValue","sceneId":scene,"fixtureId":ids[0],"attribute":key,"mode":"literal","value":value})).unwrap();
    }
    let compiled = doc.compile_scene(&scene).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let mut master = OutputMaster::default();
    master.set_percent(50).unwrap();
    let slots = compiled
        .output
        .render_with_master(player.values(), master)
        .unwrap()
        .slots;
    assert_eq!(slots[21], 128);
    assert_eq!(slots[22], 255, "不能重复衰减单元调光");
    assert_eq!(slots[26], 255);
    assert_eq!(slots[29], 255, "白光不是 RGB 合成");
    master.set_blackout(true);
    let slots = compiled
        .output
        .render_with_master(player.values(), master)
        .unwrap()
        .slots;
    assert_eq!(slots[21], 0);
    assert_eq!(slots[22], 255);
    assert_eq!(slots[29], 255);
}
#[test]
fn without_global_dimmer_each_emitter_has_its_own_intensity_mask() {
    let mut def = definition();
    def["channels"].as_array_mut().unwrap().remove(0);
    def["channels"][4]["defaultValue"] = json!(65535);
    let (doc, _, scene) = setup(def);
    assert_eq!(doc.uncontrolled_intensity_fixtures(), 0);
    let compiled = doc.compile_scene(&scene).unwrap();
    let mut master = OutputMaster::default();
    master.set_percent(50).unwrap();
    let frame = compiled
        .output
        .render_with_master(compiled.plan.defaults(), master)
        .unwrap();
    assert_eq!(frame.slots[22], 64);
    assert_eq!(frame.slots[26], 128);
    assert_eq!(frame.slots[29], 128);
}
#[test]
fn malformed_definitions_and_reserved_autonomous_channels_fail_atomically() {
    let mut doc = Document::new("拒绝试验").unwrap();
    let before = doc.clone();
    for (path, value) in [
        ("/emitters", json!([])),
        ("/emitters/1/key", json!("pattern")),
        ("/emitters/0/key", json!("pattern.bad")),
        ("/emitters/0/name", json!(" ")),
        ("/channels/1/attribute", json!("emitter.missing.dimmer")),
        (
            "/channels/1/attribute",
            json!("emitter.pattern.fixture-program"),
        ),
        ("/channels/1/attribute", json!("emitter.pattern.pan")),
        ("/channels/5/attribute", json!("emitter.wash.shutter")),
        ("/channels/1/coarse", json!(6)),
        ("/channels/1/coarse", json!(19)),
        (
            "/channels/1/defaultValue",
            json!({"functionKey":"sound.random","position":0}),
        ),
    ] {
        let mut def = definition();
        *def.pointer_mut(path).unwrap() = value;
        assert!(save(&mut doc, def).is_err(), "{path}");
        assert_eq!(doc, before);
    }
    let mut def = definition();
    def["channels"][1]["functions"] = json!([]);
    assert!(save(&mut doc, def).is_err());
    let mut def = definition();
    def.as_object_mut().unwrap().remove("emitters");
    assert!(save(&mut doc, def).is_err());
    assert_eq!(doc, before);
}
#[test]
fn missing_capability_and_forged_stored_semantics_are_rejected_on_read() {
    let (doc, _, _) = setup(definition());
    let raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let mut missing = raw.clone();
    missing["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap["key"] != "lighting.fixture-emitters");
    assert!(
        Document::decode(&serde_json::to_vec(&missing).unwrap())
            .unwrap_err()
            .contains("独立光源")
    );
    for version in [0, 2] {
        let mut bad = raw.clone();
        for cap in bad["requires"].as_array_mut().unwrap() {
            if cap["key"] == "lighting.fixture-emitters" {
                cap["version"] = json!(version);
            }
        }
        assert!(Document::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (path, value) in [
        ("/attributes/0/mix", json!("ltp")),
        ("/attributes/1/mix", json!("ltp")),
        ("/attributes/1/valueType/kind", json!("function")),
    ] {
        let mut bad = raw.clone();
        // Locate the newly added profile, independent of the built-in profile count.
        let profile = bad["lighting"]["profiles"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        *profile.pointer_mut(path).unwrap() = value;
        assert!(Document::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}

#[test]
fn emitter_budget_and_absent_metadata_do_not_reinterpret_legacy_profiles() {
    let old = Document::new("旧工程").unwrap();
    assert!(old.view().profiles.iter().all(|p| p.emitters.is_none()));
    assert_eq!(old, Document::decode(&old.encode().unwrap()).unwrap());
    let mut def = definition();
    def["emitters"] = json!(
        (0..32)
            .map(|i| json!({"key":format!("unit{i}"),"name":format!("光源 {i}")}))
            .collect::<Vec<_>>()
    );
    def["channels"]=json!((0..32).map(|i|json!({"attribute":format!("emitter.unit{i}.dimmer"),"coarse":i+1,"fine":null,"defaultValue":0})).collect::<Vec<_>>());
    def["footprint"] = json!(32);
    let mut doc = Document::new("预算").unwrap();
    save(&mut doc, def.clone()).unwrap();
    def["emitters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"extra","name":"超限"}));
    let before = doc.clone();
    assert!(save(&mut doc, def).is_err());
    assert_eq!(doc, before);
}
