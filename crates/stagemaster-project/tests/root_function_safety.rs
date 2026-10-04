#[path = "support/fixture_function.rs"]
mod support;
use serde_json::{Value, json};
use sha2::Digest;
use stagemaster_project::{FunctionSelection, ManualSceneReading, ProfileDefault};
use support::{choose, decode, edit, raw, setup};

fn with_blocked(
    attribute: &str,
    key: &str,
    mode: &str,
) -> (stagemaster_project::Document, String, Vec<String>) {
    let (doc, fixture, scenes) = setup(false);
    let mut root = raw(&doc);
    let profile = root["lighting"]["profiles"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    let channel = profile["channels"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["attribute"] == attribute)
        .unwrap();
    // Keep the safe default, replace the table's last function with a known unsafe/unknown one.
    *channel["functions"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap() = json!({
        "key":key,"name":"仅作禁用资料","mode":mode,"dmxFrom":140,"dmxTo":255,"dmxDefault":140
    });
    (decode(&root).unwrap(), fixture, scenes)
}
fn selection(key: &str, position: u16) -> ProfileDefault {
    ProfileDefault::Function(FunctionSelection {
        function_key: key.into(),
        position,
    })
}
fn prohibited() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("color-wheel", "automatic", "range"),
        ("color-wheel", "unclassified", "range"),
        ("color-wheel", "slot-sound-1", "slot"),
        ("color-wheel", "auto.1", "slot"),
        ("gobo-wheel", "random", "range"),
        ("gobo-wheel", "unclassified", "range"),
        ("gobo-wheel", "reset", "slot"),
        ("shutter", "sound", "slot"),
        ("shutter", "unclassified", "range"),
        ("prism", "macro", "slot"),
        ("prism", "unclassified", "range"),
    ]
}
#[test]
fn blocked_root_functions_refuse_edits_batches_manual_output_and_capture() {
    for (attribute, key, mode) in prohibited() {
        let (mut doc, fixture, scenes) = with_blocked(attribute, key, mode);
        let before = doc.clone();
        let output = doc.compile_scene(&scenes[0]).unwrap().output;
        let layout = format!("{:x}", sha2::Sha256::digest(doc.encode().unwrap()));
        for position in [0, if mode == "range" { 65535 } else { 0 }] {
            assert!(
                choose(&mut doc, &scenes[0], &fixture, attribute, key, position).is_err(),
                "{attribute}/{key}"
            );
            assert_eq!(doc, before);
            assert!(edit(&mut doc, json!({"op":"batch","commands":[
                {"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":"dimmer","mode":"literal","value":12000},
                {"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":attribute,"selection":{"functionKey":key,"position":position}}
            ]})).is_err());
            assert_eq!(doc, before);
            assert!(
                output
                    .manual_value(&fixture, attribute, Some(&selection(key, position)))
                    .is_err(),
                "手动旁路 {attribute}/{key}"
            );
        }
        assert!(
            doc.capture_manual_scene(
                &layout,
                vec![ManualSceneReading {
                    fixture_id: fixture.clone(),
                    attribute: attribute.into(),
                    value: 140 * 257
                }]
            )
            .is_err(),
            "记录旁路 {attribute}/{key}"
        );
        assert!(
            output
                .manual_value(
                    &fixture,
                    attribute,
                    Some(&ProfileDefault::Normalized(140 * 257))
                )
                .is_err()
        );
        assert_eq!(
            output.manual_value(&fixture, attribute, None).unwrap().1,
            None
        );
    }
}
#[test]
fn unsafe_defaults_literals_and_presets_fail_read_without_silent_migration() {
    let (mut doc, fixture, scenes) = with_blocked("color-wheel", "automatic", "range");
    choose(&mut doc, &scenes[0], &fixture, "color-wheel", "red", 0).unwrap();
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"recordPreset","name":"合法红色",
        "sceneId":scenes[0],"fixtureIds":[fixture],"attributes":["color-wheel"]}}),
    )
    .unwrap();
    for position in [0, 65535] {
        for source in ["default", "scene", "preset"] {
            let mut root = raw(&doc);
            let unsafe_value =
                json!({"kind":"function","functionKey":"automatic","position":position});
            let target = match source {
                "default" => root["lighting"]["profiles"]
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()["attributes"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|a| a["key"] == "color-wheel")
                    .unwrap()
                    .get_mut("default")
                    .unwrap(),
                "scene" => root["lighting"]["scenes"][0]["assignments"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|a| a["target"]["attribute"] == "color-wheel")
                    .unwrap()["source"]
                    .get_mut("value")
                    .unwrap(),
                _ => root["lighting"]["presets"][0]["values"][0]
                    .get_mut("value")
                    .unwrap(),
            };
            *target = unsafe_value;
            assert!(decode(&root).unwrap_err().contains("已屏蔽"), "{source}");
        }
    }
    let bytes = doc.encode().unwrap();
    assert_eq!(
        decode(&serde_json::from_slice::<Value>(&bytes).unwrap()).unwrap(),
        doc
    );
    let profile = doc.view().profiles.last().unwrap().id.clone();
    let file = doc.profile_file(&profile).unwrap();
    assert!(
        serde_json::to_value(file.definition()).unwrap()["channels"][1]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["key"] == "automatic")
    );
}

#[test]
fn legacy_numeric_function_channels_cannot_bypass_semantic_policy() {
    let (doc, _, _) = setup(false);
    for key in ["color-wheel", "gobo-wheel", "shutter", "prism"] {
        let mut root = raw(&doc);
        let profile = root["lighting"]["profiles"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        let attribute = profile["attributes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["key"] == key)
            .unwrap();
        attribute["valueType"]["kind"] = json!("normalized");
        attribute["default"] = json!({"kind":"normalized","value":65535});
        let channel = profile["channels"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["attribute"] == key)
            .unwrap();
        channel.as_object_mut().unwrap().remove("functions");
        for scene in root["lighting"]["scenes"].as_array_mut().unwrap() {
            for assignment in scene["assignments"].as_array_mut().unwrap() {
                if assignment["target"]["attribute"] == key {
                    assignment["source"]["value"] = json!({"kind":"normalized","value":65535});
                }
            }
        }
        assert!(decode(&root).is_err(), "旧百分比旁路 {key}");
    }
}

#[test]
fn explicit_controlled_shake_and_strobe_preserve_native_endpoints_and_packages() {
    use stagemaster_playback::{OutputMaster, Player};
    use stagemaster_project::PackageSelection;
    for (attribute, key) in [("gobo-wheel", "shake-dots"), ("shutter", "strobe-slow")] {
        for fine in [false, true] {
            let (doc, fixture, scenes) = with_blocked(attribute, key, "range");
            let mut root = raw(&doc);
            root["lighting"]["patches"][0]["address"] = json!(17);
            if fine {
                let profile = root["lighting"]["profiles"]
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap();
                profile["footprint"] = json!(6);
                let channel = profile["channels"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| c["attribute"] == attribute)
                    .unwrap();
                channel["encoding"] = json!("u16-be");
                channel["offsets"] = json!([5, if attribute == "gobo-wheel" { 2 } else { 3 }]);
                for f in channel["functions"].as_array_mut().unwrap() {
                    for field in ["dmxFrom", "dmxTo", "dmxDefault"] {
                        f[field] = json!(f[field].as_u64().unwrap() * 257);
                    }
                }
            }
            let mut doc = decode(&root).unwrap();
            for (position, native) in [(0, 140_u16), (65535, 255)] {
                choose(&mut doc, &scenes[0], &fixture, attribute, key, position).unwrap();
                let c = doc.compile_scene(&scenes[0]).unwrap();
                let mut p = Player::new(c.plan.clone(), 0);
                p.execute(0, 0).unwrap();
                let mut master = OutputMaster::default();
                master.set_blackout(true);
                let frame = c.output.render_with_master(p.values(), master).unwrap();
                let offset = if attribute == "gobo-wheel" { 2 } else { 3 };
                assert_eq!(frame.slots[16 + offset], u8::try_from(native).unwrap());
                if fine {
                    assert_eq!(frame.slots[21], u8::try_from(native).unwrap());
                }
                assert_eq!(
                    c.output
                        .manual_value(&fixture, attribute, Some(&selection(key, position)))
                        .unwrap()
                        .1,
                    Some(native * 257)
                );
                let package = doc
                    .build_package(&[PackageSelection::Scene {
                        id: scenes[0].clone(),
                    }])
                    .unwrap();
                let archive = stagemaster_package::Archive::open(package.bytes.as_slice()).unwrap();
                assert_eq!(
                    archive.load(package.bytes.as_slice(), 0).unwrap().plan,
                    c.plan
                );
            }
        }
    }
}
