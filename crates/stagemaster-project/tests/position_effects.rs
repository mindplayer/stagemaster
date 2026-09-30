#[path = "support/position_effect.rs"]
mod support;
use serde_json::json;
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
use support::{decode, edit, motion, put, raw, set, setup};

#[test]
fn linked_presets_recompile_centers_without_mutating_previous_plan() {
    let (mut doc, scene, ids) = setup(true);
    set(&mut doc, &scene, &ids[0], "pan", 40000);
    edit(&mut doc,json!({"op":"library","command":{"kind":"recordPreset","name":"位置","sceneId":scene,"fixtureIds":ids,"attributes":["pan","tilt"]}})).unwrap();
    let preset = doc.view().presets[0].id.clone();
    edit(&mut doc, json!({"op":"addScene","name":"引用"})).unwrap();
    let target = doc.view().scenes[1].id.clone();
    edit(&mut doc,json!({"op":"library","command":{"kind":"applyPreset","id":preset,"sceneId":target,"fixtureIds":ids,"attributes":["pan","tilt"],"linked":true}})).unwrap();
    put(&mut doc, &target, &motion(&ids)).unwrap();
    let old = doc.compile_scene(&target).unwrap().plan;
    let unchanged = old.clone();
    set(&mut doc, &scene, &ids[0], "pan", 45000);
    edit(&mut doc,json!({"op":"library","command":{"kind":"updatePreset","id":preset,"sceneId":scene,"fixtureIds":ids,"attributes":["pan","tilt"],"mode":"existing"}})).unwrap();
    let new = doc.compile_scene(&target).unwrap().plan;
    assert_ne!(old.effects()[0][0], new.effects()[0][0]);
    assert_eq!(old.effects()[0][1], new.effects()[0][1]);
    assert_eq!(old, unchanged);
}

#[test]
fn generated_frames_count_toward_existing_capacity_before_plan_allocation() {
    let (mut doc, scene, _) = setup(true);
    let v = doc.view();
    edit(&mut doc,json!({"op":"addFixture","name":"灯 11","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":11})).unwrap();
    let ids = doc
        .view()
        .fixtures
        .into_iter()
        .map(|f| f.id)
        .collect::<Vec<_>>();
    put(&mut doc, &scene, &motion(&ids)).unwrap();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"容量","sceneId":scene}}),
    )
    .unwrap();
    let mut root = raw(&doc);
    let sequence = &mut root["lighting"]["sequences"][0];
    let template = sequence["steps"][0].clone();
    sequence["steps"] = (0..683)
        .map(|i| {
            let mut step = template.clone();
            step["id"] = json!(format!("39999999-0000-4000-8001-{i:012x}"));
            step["number"] = json!((i + 1).to_string());
            step
        })
        .collect();
    let id = sequence["id"].as_str().unwrap().to_owned();
    assert!(
        decode(&root)
            .compile_sequence(&id)
            .err()
            .unwrap()
            .contains("关键帧超出计划容量")
    );
    root["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(
        decode(&root)
            .compile_sequence(&id)
            .unwrap()
            .plan
            .keyframe_count(),
        682 * 192
    );
}

#[test]
fn relative_physical_axes_quantization_and_portable_frames_match_across_a_full_cycle() {
    for fine in [true, false] {
        let (mut doc, scene, ids) = setup(fine);
        let view = doc.view();
        let model = view.fixtures[0].positioning.as_ref().unwrap();
        for (i, id) in ids.iter().enumerate() {
            set(
                &mut doc,
                &scene,
                id,
                "pan",
                model
                    .pan
                    .encode(if i == 0 { -40.0 } else { 70.0 }, fine)
                    .unwrap(),
            );
            set(
                &mut doc,
                &scene,
                id,
                "tilt",
                model
                    .tilt
                    .encode(if i == 0 { -20.0 } else { 30.0 }, fine)
                    .unwrap(),
            );
        }
        // Zero correction belongs to physical beam interpretation, never added to relative DMX twice.
        edit(&mut doc,json!({"op":"position","command":{"op":"calibrate","fixtureId":ids[0],"correction":{"panDegrees":"3","tiltDegrees":"-2"}}})).unwrap();
        let base = doc.compile_scene(&scene).unwrap().plan.steps()[0]
            .target
            .clone();
        put(&mut doc, &scene, &motion(&ids)).unwrap();
        let reopened = Document::decode(&doc.encode().unwrap()).unwrap();
        let compiled = reopened.compile_scene(&scene).unwrap();
        assert_eq!(compiled.plan.keyframe_count(), 128);
        let package = doc
            .build_package(&[PackageSelection::Scene { id: scene }])
            .unwrap();
        let archive = stagemaster_package::Archive::open(package.bytes.as_slice()).unwrap();
        let program = archive.load(package.bytes.as_slice(), 0).unwrap();
        assert_eq!(compiled.plan, program.plan);
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        for ms in 0_u32..=4096 {
            player.advance(u64::from(ms)).unwrap();
            let out = compiled.output.render(player.values()).unwrap();
            let mut slots = [0; 512];
            program.output.render(player.values(), &mut slots).unwrap();
            assert_eq!(out.slots, slots);
            for (i, fixture) in out.fixtures.iter().enumerate() {
                assert_eq!(fixture.attributes[0].value, 12345);
                for (j, axis, amplitude, offset, phase) in [
                    (1, &model.pan, 30.0, 5.0, 0.0),
                    (2, &model.tilt, 15.0, -3.0, std::f64::consts::FRAC_PI_2),
                ] {
                    let actual = axis.decode(fixture.attributes[j].value, fine).unwrap();
                    let center = axis.decode(base[i * 3 + j], fine).unwrap() + offset;
                    let expected = center
                        + amplitude
                            * (f64::from(ms) * std::f64::consts::TAU / 4096.0 - phase).sin();
                    let span = axis.max_degrees.parse::<f64>().unwrap()
                        - axis.min_degrees.parse::<f64>().unwrap();
                    let error =
                        amplitude * 0.00482 + span / if fine { 65535.0 } else { 255.0 } * 1.5;
                    assert!(
                        (actual - expected).abs() <= error,
                        "fine={fine}, t={ms}, actual={actual}, expected={expected}"
                    );
                }
            }
        }
    }
}

#[test]
fn saved_order_phase_reverse_pause_and_disabled_static_restore() {
    let (mut doc, scene, mut ids) = setup(true);
    ids.reverse();
    let mut e = motion(&ids);
    e["spreadDegrees"] = json!(360);
    e["phaseDegrees"] = json!(17);
    put(&mut doc, &scene, &e).unwrap();
    let plan = doc.compile_scene(&scene).unwrap().plan;
    let mut a = Player::new(plan.clone(), 0);
    a.execute(0, 0).unwrap();
    a.pause(1000).unwrap();
    let paused = a.values().to_vec();
    a.advance(10000).unwrap();
    assert_eq!(a.values(), paused);
    a.resume(10000).unwrap();
    a.advance(10500).unwrap();
    let mut expected = Player::new(plan.clone(), 0);
    expected.execute(0, 0).unwrap();
    expected.advance(1500).unwrap();
    assert_eq!(a.values(), expected.values());
    e["reverse"] = json!(true);
    put(&mut doc, &scene, &e).unwrap();
    let reversed = doc.compile_scene(&scene).unwrap().plan;
    assert_eq!(plan.effects()[0][0].phase, reversed.effects()[0][2].phase);
    assert_eq!(plan.effects()[0][2].phase, reversed.effects()[0][0].phase);
    e["enabled"] = json!(false);
    put(&mut doc, &scene, &e).unwrap();
    let disabled = doc.compile_scene(&scene).unwrap().plan;
    assert_eq!(disabled.keyframe_count(), 0);
    let target = disabled.steps()[0].target.clone();
    let mut p = Player::new(disabled, 0);
    p.execute(0, 0).unwrap();
    p.advance(2000).unwrap();
    assert_eq!(p.values(), target);
}

#[test]
fn invalid_motion_is_atomic_and_unreachable_stroke_is_located_without_clipping() {
    let (mut doc, scene, ids) = setup(true);
    let e = motion(&ids);
    put(&mut doc, &scene, &e).unwrap();
    let before = doc.clone();
    for (path, bad) in [
        ("/channels/0/amplitudeDegrees", json!("-1")),
        ("/channels/0/offsetDegrees", json!("3601")),
        ("/waveform", json!("triangle")),
        ("/channels/0/phaseDegrees", json!(360)),
        ("/channels/0/attribute", json!("dimmer")),
    ] {
        let mut invalid = e.clone();
        *invalid.pointer_mut(path).unwrap() = bad;
        assert!(put(&mut doc, &scene, &invalid).is_err());
        assert_eq!(doc, before);
    }
    let mut duplicate = e.clone();
    duplicate["id"] = json!("39999999-0000-4000-8000-000000000002");
    assert!(
        put(&mut doc, &scene, &duplicate)
            .unwrap_err()
            .contains("同一灯具属性")
    );
    assert_eq!(doc, before);
    set(&mut doc, &scene, &ids[0], "pan", 0);
    let error = doc.compile_scene(&scene).err().unwrap();
    for expected in ["运动场景", "圆形", "灯 1", "水平", "机械行程"] {
        assert!(error.contains(expected), "{error}");
    }
    let mut zero = e;
    zero["channels"][0]["offsetDegrees"] = json!("0");
    zero["channels"][0]["amplitudeDegrees"] = json!("0");
    put(&mut doc, &scene, &zero).unwrap();
    assert!(doc.compile_scene(&scene).is_ok());
    let mut root = raw(&doc);
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "lighting.effects.position");
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap())
            .unwrap_err()
            .contains("能力声明")
    );
    let mut root = raw(&doc);
    root["lighting"]["profiles"][2]
        .as_object_mut()
        .unwrap()
        .remove("positioning");
    assert!(
        Document::decode(&serde_json::to_vec(&root).unwrap())
            .unwrap_err()
            .contains("运动模型")
    );
}

#[test]
fn tracked_steps_use_static_targets_and_release_reverts_to_defaults() {
    let (mut doc, scene, ids) = setup(true);
    set(&mut doc, &scene, &ids[0], "pan", 40000);
    edit(&mut doc, json!({"op":"addScene","name":"继承"})).unwrap();
    let second = doc.view().scenes[1].id.clone();
    put(&mut doc, &scene, &motion(&ids)).unwrap();
    let mut other = motion(&ids);
    other["id"] = json!("39999999-0000-4000-8000-000000000002");
    put(&mut doc, &second, &other).unwrap();
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"跟踪","sceneId":scene}}),
    )
    .unwrap();
    let mut root = raw(&doc);
    root["lighting"]["scenes"][1]["assignments"]
        .as_array_mut()
        .unwrap()
        .retain(|a| a["target"]["attribute"] == "dimmer");
    let sequence = &mut root["lighting"]["sequences"][0];
    sequence["tracking"] = json!("inherited");
    let mut step = sequence["steps"][0].clone();
    step["id"] = json!("39999999-0000-4000-8000-000000000003");
    step["number"] = json!("2");
    step["sceneId"] = json!(second);
    sequence["steps"].as_array_mut().unwrap().push(step);
    let id = sequence["id"].as_str().unwrap().to_owned();
    let d = decode(&root);
    let plan = d.compile_sequence(&id).unwrap().plan;
    assert_eq!(plan.steps()[0].target, plan.steps()[1].target);
    assert_eq!(plan.effects()[0], plan.effects()[1]);
    let standalone = d.compile_scene(&second).unwrap().plan;
    assert_ne!(plan.effects()[1], standalone.effects()[0]);
    root["lighting"]["scenes"][1]["assignments"] =
        json!([{"target":{"fixtureId":ids[0],"attribute":"pan"},"operation":"release"}]);
    let released = decode(&root).compile_sequence(&id).unwrap().plan;
    assert_eq!(released.effects()[1], standalone.effects()[0]);
}
