use super::{choose, decode, definition, raw, setup, support};
use serde_json::json;
use sha2::Digest;
use stagemaster_project::{FunctionSelection, ManualSceneReading, ProfileDefault};

#[test]
fn manual_output_and_capture_cannot_bypass_autonomous_program_blocking() {
    let (doc, fixtures, scenes) = setup(definition());
    let output = doc.compile_scene(&scenes[0]).unwrap().output;
    let layout = format!("{:x}", sha2::Sha256::digest(doc.encode().unwrap()));
    for kind in ["auto", "sound"] {
        for number in 0..4 {
            let key = format!("{kind}.{number}");
            let value = ProfileDefault::Function(FunctionSelection {
                function_key: key.clone(),
                position: 0,
            });
            assert!(
                output
                    .manual_value(&fixtures[0], "fixture-program", Some(&value))
                    .is_err(),
                "手动输出不能绕过编排保护"
            );
            let def = definition();
            let slot = def["channels"][4]["functions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["key"] == key)
                .unwrap();
            let native = u16::try_from(slot["dmxDefault"].as_u64().unwrap() * 257).unwrap();
            assert!(
                doc.capture_manual_scene(
                    &layout,
                    vec![ManualSceneReading {
                        fixture_id: fixtures[0].clone(),
                        attribute: "fixture-program".into(),
                        value: native
                    }]
                )
                .is_err()
            );
        }
    }
    let external = ProfileDefault::Function(FunctionSelection {
        function_key: "external".into(),
        position: 0,
    });
    assert_eq!(
        output
            .manual_value(&fixtures[0], "fixture-program", Some(&external))
            .unwrap()
            .1,
        Some(0)
    );
    assert_eq!(
        output
            .manual_value(&fixtures[0], "fixture-program", None)
            .unwrap()
            .1,
        None
    );
    assert!(
        output
            .manual_value(
                &fixtures[0],
                "fixture-program",
                Some(&ProfileDefault::Normalized(65535))
            )
            .is_err()
    );
}

#[test]
fn autonomous_slots_are_blocked_at_core_edit_even_if_present_in_the_manual_table() {
    let (mut doc, fixtures, scenes) = setup(definition());
    let before = doc.clone();
    for kind in ["auto", "sound"] {
        for number in 0..4 {
            assert!(
                choose(
                    &mut doc,
                    &scenes[0],
                    &fixtures[0],
                    &format!("{kind}.{number}"),
                    0
                )
                .is_err(),
                "自主档位必须由 Rust 拒绝，不能只隐藏 UI"
            );
            assert_eq!(doc, before);
        }
    }
}

#[test]
fn autonomous_scene_or_preset_payloads_and_mixed_batches_are_rejected_before_publication() {
    let (mut doc, fixtures, scenes) = setup(definition());
    choose(&mut doc, &scenes[0], &fixtures[0], "external", 0).unwrap();
    support::edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"recordPreset","name":"外部控制",
        "sceneId":scenes[0],"fixtureIds":[fixtures[0]],"attributes":["fixture-program"]}}),
    )
    .unwrap();
    let before = doc.clone();
    for kind in ["auto", "sound"] {
        for number in 0..4 {
            let key = format!("{kind}.{number}");
            let mut scene_root = raw(&doc);
            let entry = scene_root["lighting"]["scenes"][0]["assignments"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["target"]["attribute"] == "fixture-program")
                .unwrap();
            entry["source"] = json!({"kind":"literal","value":{"kind":"function","functionKey":key,"position":0}});
            let error = decode(&scene_root).unwrap_err();
            assert!(error.contains("已屏蔽"), "{error}");
            let mut preset_root = raw(&doc);
            let entry = preset_root["lighting"]["presets"][0]["values"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["target"]["attribute"] == "fixture-program")
                .unwrap();
            entry["value"]["functionKey"] = json!(key);
            let error = decode(&preset_root).unwrap_err();
            assert!(error.contains("已屏蔽"), "{error}");
            assert!(support::edit(&mut doc, json!({"op":"batch","commands":[
                {"op":"setSceneFunctionValue","sceneId":scenes[1],"fixtureId":fixtures[0],"attribute":"fixture-program","selection":{"functionKey":"external","position":0}},
                {"op":"setSceneFunctionValue","sceneId":scenes[1],"fixtureId":fixtures[1],"attribute":"fixture-program","selection":{"functionKey":key,"position":0}}
            ]})).unwrap_err().contains("已屏蔽"));
            assert_eq!(doc, before, "有效+禁用批次不能部分修改");
        }
    }
}
