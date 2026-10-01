#[path = "support/fixture_function.rs"]
mod support;
use serde_json::json;
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
use support::{choose, decode, definition, edit, raw, setup};

#[test]
fn typed_functions_persist_and_compile_with_exact_native_mapping_and_versioned_package() {
    for fine in [false, true] {
        let (mut doc, id, scenes) = setup(fine);
        choose(&mut doc, &scenes[0], &id, "color-wheel", "red", 0).unwrap();
        choose(&mut doc, &scenes[0], &id, "shutter", "strobe", 32768).unwrap();
        let reopened = decode(&raw(&doc)).unwrap();
        assert_eq!(reopened, doc);
        let view = reopened.view();
        let selected = &view.scenes[0].values;
        assert_eq!(
            selected
                .iter()
                .find(|v| v.attribute == "color-wheel")
                .unwrap()
                .function_value
                .as_ref()
                .unwrap()
                .function_key,
            "red"
        );
        let c = reopened.compile_scene(&scenes[0]).unwrap();
        assert_eq!(c.plan.snap_attributes(), [1, 2, 3, 4]);
        let mut p = Player::new(c.plan.clone(), 0);
        p.execute(0, 0).unwrap();
        let out = c.output.render(p.values()).unwrap();
        assert_eq!(out.slots[1], if fine { 0 } else { 20 });
        if fine {
            assert_eq!(out.slots[5], 20);
        }
        assert_eq!(out.slots[3], 136);
        let built = doc
            .build_package(&[PackageSelection::Scene {
                id: scenes[0].clone(),
            }])
            .unwrap();
        let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
        assert_eq!(archive.semantics(), 2);
        assert_eq!(
            archive.load(built.bytes.as_slice(), 0).unwrap().plan,
            c.plan
        );
    }
}
#[test]
fn scene_transition_never_traverses_other_wheel_functions_or_modifies_dimmer_fade() {
    let (mut doc, id, scenes) = setup(false);
    for (scene, key, dim) in [(&scenes[0], "red", 10000), (&scenes[1], "blue", 50000)] {
        choose(&mut doc, scene, &id, "color-wheel", key, 0).unwrap();
        edit(
            &mut doc,
            json!({"op":"setSceneValue","sceneId":scene,"fixtureId":id,
            "attribute":"dimmer","mode":"literal","value":dim}),
        )
        .unwrap();
    }
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"切换","sceneId":scenes[0]}}),
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
    let c = doc.compile_sequence(&doc.view().sequences[0].id).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    p.execute(1, 0).unwrap();
    for t in 0..=150 {
        p.advance(t).unwrap();
        assert_eq!(p.values()[1], if t < 50 { 20 * 257 } else { 40 * 257 });
        if t == 100 {
            assert_eq!(p.values()[0], 30000);
        }
    }
}
#[test]
fn invalid_profile_values_and_atomic_batches_do_not_rewrite_the_project() {
    let (mut doc, id, scenes) = setup(false);
    let before = doc.clone();
    for (attribute, key, pos) in [
        ("shutter", "absent", 0),
        ("color-wheel", "red", 1),
        ("dimmer", "red", 0),
    ] {
        assert!(choose(&mut doc, &scenes[0], &id, attribute, key, pos).is_err());
        assert_eq!(doc, before);
    }
    assert!(
        edit(
            &mut doc,
            json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":id,
        "attribute":"color-wheel","mode":"literal","value":50000})
        )
        .is_err()
    );
    assert_eq!(doc, before);
    assert!(edit(&mut doc,json!({"op":"batch","commands":[
        {"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":id,"attribute":"color-wheel","selection":{"functionKey":"red","position":0}},
        {"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":id,"attribute":"prism","selection":{"functionKey":"invalid","position":0}}
    ]})).is_err());
    assert_eq!(doc, before);
    for (pointer, value) in [
        ("/channels/1/functions/1/dmxFrom", json!(15)),
        ("/channels/1/defaultValue/functionKey", json!("missing")),
        ("/channels/1/defaultValue", json!(100)),
        ("/channels/1/functions", json!([])),
        ("/channels/1/attribute", json!("control")),
    ] {
        let mut d = definition(false);
        *d.pointer_mut(pointer).unwrap() = value;
        assert!(
            edit(
                &mut doc,
                json!({"op":"fixture","command":{"op":"saveProfile","definition":d}})
            )
            .is_err()
        );
        assert_eq!(doc, before);
    }
    let mut root = raw(&doc);
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["key"] != "lighting.fixture-functions");
    assert!(decode(&root).unwrap_err().contains("能力声明"));
}
#[test]
fn preset_reference_update_unlink_and_release_retain_function_identity() {
    let (mut doc, id, scenes) = setup(false);
    choose(&mut doc, &scenes[0], &id, "color-wheel", "red", 0).unwrap();
    edit(&mut doc,json!({"op":"library","command":{"kind":"recordPreset","name":"轮盘","sceneId":scenes[0],"fixtureIds":[id],"attributes":["color-wheel"]}})).unwrap();
    let preset = doc.view().presets[0].id.clone();
    edit(&mut doc,json!({"op":"library","command":{"kind":"applyPreset","id":preset,"sceneId":scenes[1],"fixtureIds":[id],"attributes":["color-wheel"],"linked":true}})).unwrap();
    let old = doc.compile_scene(&scenes[1]).unwrap().plan;
    choose(&mut doc, &scenes[0], &id, "color-wheel", "blue", 0).unwrap();
    edit(&mut doc,json!({"op":"library","command":{"kind":"updatePreset","id":preset,"sceneId":scenes[0],"fixtureIds":[id],"attributes":["color-wheel"],"mode":"existing"}})).unwrap();
    let new = doc.compile_scene(&scenes[1]).unwrap().plan;
    assert_eq!(old.steps()[0].target[1], 20 * 257);
    assert_eq!(new.steps()[0].target[1], 40 * 257);
    edit(&mut doc,json!({"op":"library","command":{"kind":"detach","sceneId":scenes[1],"fixtureIds":[id],"attributes":["color-wheel"]}})).unwrap();
    assert_eq!(doc.compile_scene(&scenes[1]).unwrap().plan, new);
    assert_eq!(
        doc.view().scenes[1]
            .values
            .iter()
            .find(|v| v.attribute == "color-wheel")
            .unwrap()
            .function_value
            .as_ref()
            .unwrap()
            .function_key,
        "blue"
    );
    edit(&mut doc,json!({"op":"setSceneValue","sceneId":scenes[1],"fixtureId":id,"attribute":"color-wheel","mode":"release","value":0})).unwrap();
    assert_eq!(
        doc.compile_scene(&scenes[1]).unwrap().plan.steps()[0].target[1],
        0
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
