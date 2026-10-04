#[path = "support/fixture_program_blocking.rs"]
mod blocking;
#[path = "support/fixture_program.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_playback::{OutputMaster, Player};
use stagemaster_project::{Document, PackageSelection, ProfileFile};
use support::{choose, decode, definition, raw, save, setup};

#[test]
fn internal_program_has_neutral_default_and_all_nine_explicit_slots_roundtrip() {
    let mut doc = Document::new("内置程序验收").unwrap();
    save(&mut doc, definition()).unwrap();
    let view = doc.view();
    let profile = view.profiles.last().unwrap();
    assert!(profile.authorable);
    let file = doc.profile_file(&profile.id).unwrap().encode().unwrap();
    assert_eq!(
        serde_json::to_value(ProfileFile::decode(&file).unwrap().definition()).unwrap(),
        definition()
    );
    let bytes = doc.encode().unwrap();
    assert_eq!(Document::decode(&bytes).unwrap(), doc);
    let root: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        root["requires"]
            .as_array()
            .unwrap()
            .contains(&json!({"key":"lighting.fixture-programs","version":1}))
    );
    save(
        &mut doc,
        serde_json::to_value(ProfileFile::decode(&file).unwrap().definition()).unwrap(),
    )
    .unwrap();
    let imported = doc.view();
    let pair = &imported.profiles[imported.profiles.len() - 2..];
    assert_ne!(pair[0].id, pair[1].id);
    assert_ne!(pair[0].revision, pair[1].revision);
    assert_eq!(
        serde_json::to_value(&pair[0].channels).unwrap(),
        serde_json::to_value(&pair[1].channels).unwrap()
    );
}

#[test]
fn neutral_precision_endpoints_encode_exactly_and_all_eight_autonomous_slots_are_blocked() {
    for fine in [false, true] {
        for representative_at_end in [false, true] {
            let mut def = definition();
            if fine {
                def["channels"][4]["fine"] = json!(6);
            }
            for f in def["channels"][4]["functions"].as_array_mut().unwrap() {
                if fine {
                    for field in ["dmxFrom", "dmxTo", "dmxDefault"] {
                        f[field] = json!(f[field].as_u64().unwrap() * 257);
                    }
                }
                f["dmxDefault"] = f[if representative_at_end {
                    "dmxTo"
                } else {
                    "dmxFrom"
                }]
                .clone();
            }
            let (mut doc, fixtures, scenes) = setup(def.clone());
            for f in def["channels"][4]["functions"].as_array().unwrap() {
                if f["key"] != "external" {
                    let before = doc.clone();
                    assert!(
                        choose(
                            &mut doc,
                            &scenes[0],
                            &fixtures[0],
                            f["key"].as_str().unwrap(),
                            0
                        )
                        .is_err()
                    );
                    assert_eq!(doc, before);
                    continue;
                }
                choose(
                    &mut doc,
                    &scenes[0],
                    &fixtures[0],
                    f["key"].as_str().unwrap(),
                    0,
                )
                .unwrap();
                let c = doc.compile_scene(&scenes[0]).unwrap();
                assert_eq!(c.plan.snap_attributes(), [4, 9]);
                let mut p = Player::new(c.plan, 0);
                p.execute(0, 0).unwrap();
                let mut master = OutputMaster::default();
                master.set_percent(37).unwrap();
                master.set_blackout(true);
                let slots = c
                    .output
                    .render_with_master(p.values(), master)
                    .unwrap()
                    .slots;
                let native = u16::try_from(f["dmxDefault"].as_u64().unwrap()).unwrap();
                assert_eq!(
                    slots[25],
                    if fine {
                        native.to_be_bytes()[0]
                    } else {
                        u8::try_from(native).unwrap()
                    }
                );
                assert_eq!(slots[21], if fine { native.to_be_bytes()[1] } else { 0 });
                assert_eq!(&slots[16..20], &[0x12, 0x34, 0xab, 0xcd]);
                assert_eq!(slots[23], 0);
                assert_eq!(slots[24], 0x34);
                assert_eq!(slots[26], 0, "复位通道始终未映射");
                let external = u16::try_from(
                    def["channels"][4]["functions"][0]["dmxDefault"]
                        .as_u64()
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(
                    slots[36],
                    if fine {
                        external.to_be_bytes()[0]
                    } else {
                        u8::try_from(external).unwrap()
                    }
                );
                assert_eq!(slots[32], if fine { external.to_be_bytes()[1] } else { 0 });
                assert!(slots[..16].iter().chain(&slots[38..]).all(|v| *v == 0));
            }
        }
    }
}

#[test]
fn fades_and_existing_semantics_two_package_never_enter_blocked_program_ranges() {
    let (mut doc, fixtures, scenes) = setup(definition());
    for (scene, key, dim) in [
        (&scenes[0], "external", 10000),
        (&scenes[1], "external", 50000),
    ] {
        choose(&mut doc, scene, &fixtures[0], key, 0).unwrap();
        support::edit(
            &mut doc,
            json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixtures[0],
            "attribute":"dimmer","mode":"literal","value":dim}),
        )
        .unwrap();
    }
    support::edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"程序切换","sceneId":scenes[0]}}),
    )
    .unwrap();
    let mut root = raw(&doc);
    let mut first = root["lighting"]["sequences"][0]["steps"][0].clone();
    first["fade"] = json!({"ticks":"0","ticksPerSecond":"1000"});
    let mut second = first.clone();
    second["id"] = json!("99999999-0000-4000-8000-000000000001");
    second["number"] = json!("2");
    second["sceneId"] = json!(scenes[1]);
    second["delay"] = json!({"ticks":"50","ticksPerSecond":"1000"});
    second["fade"] = json!({"ticks":"100","ticksPerSecond":"1000"});
    root["lighting"]["sequences"][0]["steps"] = json!([first, second]);
    let doc = decode(&root).unwrap();
    let id = doc.view().sequences[0].id.clone();
    let c = doc.compile_sequence(&id).unwrap();
    let built = doc
        .build_package(&[PackageSelection::Sequence { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    assert_eq!(archive.semantics(), 2);
    assert_eq!(
        archive.load(built.bytes.as_slice(), 0).unwrap().plan,
        c.plan
    );
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    p.execute(1, 0).unwrap();
    for t in 0..=150 {
        p.advance(t).unwrap();
        assert_eq!(p.values()[4], 0, "连续渐变不得经过任何禁用档位");
        if t == 100 {
            assert_eq!(p.values()[2], 30000);
        }
    }
    p.stop(150).unwrap();
    assert_eq!(p.values()[4], 0);
}

#[test]
fn presets_release_and_removal_preserve_semantic_identity_not_encoded_percentages() {
    let (mut doc, fixtures, scenes) = setup(definition());
    choose(&mut doc, &scenes[0], &fixtures[0], "external", 0).unwrap();
    support::edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"recordPreset","name":"外部控制",
        "sceneId":scenes[0],"fixtureIds":[fixtures[0]],"attributes":["fixture-program"]}}),
    )
    .unwrap();
    let preset = doc.view().presets[0].id.clone();
    support::edit(&mut doc,json!({"op":"library","command":{"kind":"applyPreset","id":preset,
        "sceneId":scenes[1],"fixtureIds":[fixtures[0]],"attributes":["fixture-program"],"linked":true}})).unwrap();
    assert_eq!(
        doc.compile_scene(&scenes[1]).unwrap().plan.steps()[0].target[4],
        0
    );
    choose(&mut doc, &scenes[0], &fixtures[0], "external", 0).unwrap();
    support::edit(&mut doc,json!({"op":"library","command":{"kind":"updatePreset","id":preset,
        "sceneId":scenes[0],"fixtureIds":[fixtures[0]],"attributes":["fixture-program"],"mode":"existing"}})).unwrap();
    assert_eq!(
        doc.view().scenes[1]
            .values
            .iter()
            .find(|v| v.attribute == "fixture-program")
            .unwrap()
            .function_value
            .as_ref()
            .unwrap()
            .function_key,
        "external"
    );
    for mode in ["release", "remove"] {
        support::edit(
            &mut doc,
            json!({"op":"setSceneValue","sceneId":scenes[1],"fixtureId":fixtures[0],
            "attribute":"fixture-program","mode":mode,"value":0}),
        )
        .unwrap();
        assert_eq!(
            doc.compile_scene(&scenes[1]).unwrap().plan.steps()[0].target[4],
            0
        );
    }
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}

#[test]
fn unsafe_authoring_and_atomic_edits_leave_original_document_unchanged() {
    let (mut doc, fixtures, scenes) = setup(definition());
    let before = doc.clone();
    for (pointer, value) in [
        ("/channels/4/defaultValue/functionKey", json!("auto.3")),
        ("/channels/4/functions/1/mode", json!("range")),
        ("/channels/4/functions/0/key", json!("auto.external")),
        ("/channels/4/functions/1/key", json!("reset")),
        ("/channels/4/functions/1/dmxFrom", json!(59)),
        ("/channels/4/functions/1/appearance", json!({"kind":"open"})),
    ] {
        let mut def = definition();
        if let Some(field) = def.pointer_mut(pointer) {
            *field = value;
        } else {
            def["channels"][4]["functions"][1]["appearance"] = value;
        }
        assert!(save(&mut doc, def).is_err(), "{pointer}");
        assert_eq!(doc, before);
    }
    for (key, position) in [("reset", 0), ("auto.3", 1)] {
        assert!(choose(&mut doc, &scenes[0], &fixtures[0], key, position).is_err());
        assert_eq!(doc, before);
    }
    assert!(support::edit(&mut doc,json!({"op":"batch","commands":[
        {"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":fixtures[0],"attribute":"fixture-program","selection":{"functionKey":"auto.3","position":0}},
        {"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixtures[1],"attribute":"fixture-program","mode":"literal","value":65535}
    ]})).is_err());
    assert_eq!(doc, before);
}

#[test]
fn unused_program_profiles_require_capability_and_typed_neutral_defaults_on_read() {
    let mut doc = Document::new("未配灯的档案").unwrap();
    save(&mut doc, definition()).unwrap();
    let original = raw(&doc);
    for key in ["lighting.fixture-programs", "lighting.fixture-functions"] {
        let mut root = original.clone();
        root["requires"]
            .as_array_mut()
            .unwrap()
            .retain(|c| c["key"] != key);
        assert!(decode(&root).unwrap_err().contains("能力声明"));
    }
    for (pointer, value) in [
        (
            "/lighting/profiles/2/attributes/4/default/functionKey",
            json!("auto.3"),
        ),
        (
            "/lighting/profiles/2/attributes/4/valueType/kind",
            json!("normalized"),
        ),
    ] {
        let mut root = original.clone();
        // New documents may add other defaults: locate our profile instead of hard-coding its index.
        let profile = root["lighting"]["profiles"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        let relative = pointer.split("/attributes/").nth(1).unwrap();
        *profile
            .pointer_mut(&format!("/attributes/{relative}"))
            .unwrap() = value;
        assert!(decode(&root).is_err());
    }
    let mut root = original;
    root["requires"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["key"] == "lighting.fixture-programs")
        .unwrap()["version"] = json!(2);
    assert!(decode(&root).is_err());
}

#[test]
fn batch_exchange_keeps_program_keys_and_rejects_incompatible_control_tables_atomically() {
    let (mut doc, fixtures, scenes) = setup(definition());
    choose(&mut doc, &scenes[0], &fixtures[0], "external", 0).unwrap();
    choose(&mut doc, &scenes[0], &fixtures[1], "external", 0).unwrap();
    let before_plan = doc.compile_scene(&scenes[0]).unwrap().plan;
    let file = doc
        .profile_file(&doc.view().profiles.last().unwrap().id)
        .unwrap();
    save(&mut doc, serde_json::to_value(file.definition()).unwrap()).unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    support::edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":fixtures,"profileId":target,"layout":null}})).unwrap();
    assert_eq!(doc.compile_scene(&scenes[0]).unwrap().plan, before_plan);
    let mut bad = definition();
    bad["channels"][4]["functions"][1]["dmxDefault"] = json!(61);
    save(&mut doc, bad).unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    assert!(support::edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":fixtures,"profileId":target,"layout":null}})).is_err());
    assert_eq!(doc, before);
    let (mut old, old_ids, _) = support::speed_setup(false);
    save(&mut old, definition()).unwrap();
    let target = old.view().profiles.last().unwrap().id.clone();
    let before = old.clone();
    assert!(support::edit(&mut old,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":old_ids,"profileId":target,"layout":null}})).is_err());
    assert_eq!(old, before, "不能偷偷给缺此属性的旧灯增加程序通道");
}
