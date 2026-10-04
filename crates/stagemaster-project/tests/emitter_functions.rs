#[path = "support/emitter_functions.rs"]
mod support;
use serde_json::json;
use stagemaster_project::{Document, FunctionSelection, ProfileDefault, ProfileFile};
use support::{definition, edit, save, setup};
const PATTERN: &str = "emitter.pattern.shutter";
const WASH: &str = "emitter.wash.shutter";
#[test]
fn two_emitter_shutters_are_typed_independent_and_roundtrip() {
    let (mut doc, ids, scene) = setup(definition());
    let view = doc.view();
    assert!(view.profiles.last().unwrap().authorable);
    assert_eq!(
        view.fixtures[0]
            .attributes
            .iter()
            .find(|a| a.key == PATTERN)
            .unwrap()
            .label,
        "图案光源 · 快门与频闪"
    );
    for (fixture, key, position) in [
        (&ids[0], PATTERN, 65535),
        (&ids[0], WASH, 0),
        (&ids[1], WASH, 32768),
    ] {
        edit(&mut doc,json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"selection":{"functionKey":"strobe","position":position}})).unwrap();
    }
    let compiled = doc.compile_scene(&scene).unwrap();
    let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let out = compiled.output.render(player.values()).unwrap();
    assert_eq!(
        [out.slots[23], out.slots[30], out.slots[41], out.slots[48]],
        [255, 16, 0, 136]
    );
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
    let file = doc
        .profile_file(&view.profiles.last().unwrap().id)
        .unwrap()
        .encode()
        .unwrap();
    assert_eq!(
        serde_json::to_value(ProfileFile::decode(&file).unwrap().definition()).unwrap(),
        definition()
    );
}

#[test]
fn invalid_autonomous_keys_modes_owners_and_numeric_shortcuts_are_atomic() {
    let mut doc = Document::new("拒绝非法单元功能").unwrap();
    let before = doc.clone();
    for key in [
        "sound-0",
        "auto-0",
        "reset",
        "random",
        "function-any",
        "strobe-other",
    ] {
        let mut def = definition();
        def["channels"][6]["functions"][1]["key"] = json!(key);
        assert!(save(&mut doc, def).is_err());
        assert_eq!(doc, before);
    }
    for (index, field, value) in [
        (6, "attribute", json!("emitter.unknown.shutter")),
        (6, "defaultValue", json!(0)),
        (6, "functions", json!([])),
        (6, "attribute", json!("emitter.pattern.fixture-program")),
    ] {
        let mut def = definition();
        def["channels"][index][field] = value;
        assert!(save(&mut doc, def).is_err());
        assert_eq!(doc, before);
    }
    for (index, row, mode) in [(6, 1, "slot"), (8, 1, "range"), (9, 2, "slot")] {
        let mut def = definition();
        def["channels"][index]["functions"][row]["mode"] = json!(mode);
        assert!(save(&mut doc, def).is_err());
        assert_eq!(doc, before);
    }
    let (mut doc, ids, scene) = setup(definition());
    let before = doc.clone();
    for command in [
        json!({"op":"setSceneValue","sceneId":scene,"fixtureId":ids[0],"attribute":PATTERN,"mode":"literal","value":0}),
        json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":ids[0],"attribute":PATTERN,"selection":{"functionKey":"open","position":1}}),
        json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":ids[0],"attribute":PATTERN,"selection":{"functionKey":"sound-0","position":0}}),
    ] {
        assert!(edit(&mut doc, command).is_err());
        assert_eq!(doc, before);
    }
    assert!(edit(&mut doc,json!({"op":"batch","commands":[
        {"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":ids[0],"attribute":PATTERN,"selection":{"functionKey":"strobe","position":65535}},
        {"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":ids[1],"attribute":WASH,"selection":{"functionKey":"random","position":0}}
    ]})).is_err());
    assert_eq!(doc, before);
}
#[test]
fn unused_profile_capability_and_forged_stored_function_rules_are_strict() {
    let mut doc = Document::new("未使用模式仍受保护").unwrap();
    save(&mut doc, definition()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for version in [None, Some(2)] {
        let mut p = raw.clone();
        let requires = p["requires"].as_array_mut().unwrap();
        let index = requires
            .iter()
            .position(|r| r["key"] == "lighting.fixture-emitter-functions")
            .unwrap();
        if let Some(version) = version {
            requires[index]["version"] = json!(version);
        } else {
            requires.remove(index);
        }
        assert!(Document::decode(&serde_json::to_vec(&p).unwrap()).is_err());
    }
    for (section, index, field, value) in [
        ("attributes", 6, "mix", json!("htp")),
        ("attributes", 6, "valueType", json!({"kind":"normalized"})),
        (
            "channels",
            6,
            "functions",
            json!([{"key":"sound-0","name":"声控","mode":"slot","dmxFrom":0,"dmxTo":255,"dmxDefault":0}]),
        ),
    ] {
        let mut p = raw.clone();
        p["lighting"]["profiles"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap()[section][index][field] = value;
        assert!(Document::decode(&serde_json::to_vec(&p).unwrap()).is_err());
    }
}
#[test]
fn fine_before_coarse_and_native_endpoints_preserve_independent_control() {
    let mut def = definition();
    def["channels"][6]["fine"] = json!(2);
    def["channels"][6]["functions"][1]["dmxTo"] = json!(65535);
    let (mut doc, ids, scene) = setup(def);
    let c = doc.compile_scene(&scene).unwrap();
    for position in [0, 1, 32768, 65535] {
        let (_, encoded) = c
            .output
            .manual_value(
                &ids[0],
                PATTERN,
                Some(&ProfileDefault::Function(FunctionSelection {
                    function_key: "strobe".into(),
                    position,
                })),
            )
            .unwrap();
        let native = 16 + ((65519 * u32::from(position) + 32767) / 65535);
        assert_eq!(encoded, Some(u16::try_from(native).unwrap()));
        edit(&mut doc, json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":ids[0],"attribute":PATTERN,"selection":{"functionKey":"strobe","position":position}})).unwrap();
        let compiled = doc.compile_scene(&scene).unwrap();
        let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        let frame = compiled.output.render(player.values()).unwrap();
        let [coarse, fine] = u16::try_from(native).unwrap().to_be_bytes();
        assert_eq!([frame.slots[23], frame.slots[17]], [coarse, fine]);
        assert_eq!([frame.slots[41], frame.slots[35]], [0, 0]);
    }
    let values = c.plan.defaults().to_vec();
    let out = c.output.render(&values).unwrap();
    assert_eq!([out.slots[23], out.slots[17]], [0, 0]);
    assert!(
        c.output
            .manual_value(
                &ids[0],
                "emitter.pattern.color-wheel",
                Some(&ProfileDefault::Function(FunctionSelection {
                    function_key: "slot-one".into(),
                    position: 1
                }))
            )
            .is_err()
    );
    assert!(
        c.output
            .manual_value(&ids[0], PATTERN, Some(&ProfileDefault::Normalized(65535)))
            .is_err()
    );
}
