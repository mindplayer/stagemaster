#[path = "support/position_effect.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
use stagemaster_spatial::{Installation, positioning::JointAngles};
use support::{decode, edit, motion, put, raw, set, setup};

fn position(doc: &mut Document, scene: &str, ids: &[String], mut c: Value) -> Result<(), String> {
    c["sceneId"] = json!(scene);
    c["fixtureIds"] = json!(ids);
    edit(doc, json!({"op":"position","command":c}))
}
fn value(doc: &Document, scene: &str, fixture: &str, key: &str) -> u16 {
    let view = doc.view();
    u16::try_from(
        view.scenes
            .iter()
            .find(|s| s.id == scene)
            .unwrap()
            .values
            .iter()
            .find(|v| v.fixture_id == fixture && v.attribute == key)
            .unwrap()
            .value
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn relative_axis_preserves_spread_untouched_attributes_and_preset_resources() {
    let (mut doc, scene, ids) = setup(true);
    set(&mut doc, &scene, &ids[0], "pan", 20000);
    set(&mut doc, &scene, &ids[1], "pan", 42000);
    edit(&mut doc,json!({"op":"library","command":{"kind":"recordPreset","name":"位置","sceneId":scene,"fixtureIds":ids,"attributes":["pan","tilt"]}})).unwrap();
    let preset = doc.view().presets[0].id.clone();
    edit(&mut doc, json!({"op":"addScene","name":"引用"})).unwrap();
    let target = doc.view().scenes[1].id.clone();
    edit(&mut doc,json!({"op":"library","command":{"kind":"applyPreset","id":preset,"sceneId":target,"fixtureIds":ids,"attributes":["pan","tilt"],"linked":true}})).unwrap();
    let before = raw(&doc);
    position(
        &mut doc,
        &target,
        &ids,
        json!({"op":"offsetAxes","panDegrees":"10","tiltDegrees":"0"}),
    )
    .unwrap();
    let after = raw(&doc);
    assert_eq!(before["lighting"]["presets"], after["lighting"]["presets"]);
    assert_eq!(
        before["lighting"]["scenes"][0],
        after["lighting"]["scenes"][0]
    );
    for id in &ids {
        let m = doc.position_model(id).unwrap().unwrap();
        let delta = m.pan.decode(value(&doc, &target, id, "pan"), true).unwrap()
            - m.pan.decode(value(&doc, &scene, id, "pan"), true).unwrap();
        assert!((delta - 10.0).abs() < 540.0 / 65535.0 / 2.0);
        for key in ["tilt", "dimmer"] {
            let assignment = |root: &Value| {
                root["lighting"]["scenes"][1]["assignments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|a| a["target"]["fixtureId"] == *id && a["target"]["attribute"] == key)
                    .cloned()
            };
            assert_eq!(assignment(&before), assignment(&after));
        }
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn group_limits_precision_effects_and_empty_deltas_are_atomic() {
    for fine in [false, true] {
        let (mut doc, scene, ids) = setup(fine);
        set(&mut doc, &scene, &ids[1], "pan", 0); // reversed pan: +270 degrees
        let before = doc.clone();
        let error = position(
            &mut doc,
            &scene,
            &ids,
            json!({"op":"offsetAxes","panDegrees":"10"}),
        )
        .unwrap_err();
        assert!(error.contains("灯 6"));
        assert_eq!(doc, before);
        for c in [
            json!({"op":"offsetAxes"}),
            json!({"op":"offsetAxes","panDegrees":"0"}),
            json!({"op":"offsetAxes","panDegrees":"NaN"}),
            json!({"op":"offsetAxes","tiltDegrees":"0.0001"}),
        ] {
            assert!(position(&mut doc, &scene, &ids, c).is_err());
            assert_eq!(doc, before);
        }
        put(&mut doc, &scene, &motion(&ids)).unwrap();
        let before = doc.clone();
        for c in [
            json!({"op":"offsetAxes","tiltDegrees":"1"}),
            json!({"op":"flip"}),
        ] {
            let error = position(&mut doc, &scene, &ids, c).unwrap_err();
            assert!(error.contains("圆形"));
            assert_eq!(doc, before);
        }
    }
    let (mut doc, scene, ids) = setup(false);
    let before = doc.clone();
    assert!(
        position(
            &mut doc,
            &scene,
            &ids,
            json!({"op":"offsetAxes","panDegrees":"0.1"})
        )
        .unwrap_err()
        .contains("精度")
    );
    assert_eq!(doc, before);
    position(
        &mut doc,
        &scene,
        &ids,
        json!({"op":"offsetAxes","panDegrees":"3"}),
    )
    .unwrap();
}
#[test]
fn unplaced_flip_handles_zero_reversal_and_quantization_without_changing_other_data() {
    for fine in [false, true] {
        let (mut doc, scene, ids) = setup(fine);
        position(
            &mut doc,
            &scene,
            &ids,
            json!({"op":"axes","panDegrees":"60","tiltDegrees":"40"}),
        )
        .unwrap();
        edit(&mut doc,json!({"op":"position","command":{"op":"calibrate","fixtureId":ids[0],"correction":{"panDegrees":"13","tiltDegrees":"-7"}}})).unwrap();
        let before = doc.clone();
        position(&mut doc, &scene, &ids, json!({"op":"flip"})).unwrap();
        for id in &ids {
            let f = doc
                .view()
                .fixtures
                .into_iter()
                .find(|f| f.id == *id)
                .unwrap();
            let m = f.positioning.unwrap();
            let head = m.head(f.zero_correction.as_ref()).unwrap();
            let angles = |doc: &Document| JointAngles {
                pan_degrees: m.pan.decode(value(doc, &scene, id, "pan"), fine).unwrap(),
                tilt_degrees: m.tilt.decode(value(doc, &scene, id, "tilt"), fine).unwrap(),
            };
            let install = Installation {
                position_meters: [1.0, 2.0, 6.0],
                rotation_degrees_xyz: [180.0, 0.0, 23.0],
            };
            let a = head.ray(install, angles(&before)).unwrap();
            let b = head.ray(install, angles(&doc)).unwrap();
            let error = a
                .direction
                .into_iter()
                .zip(b.direction)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(error < if fine { 0.00011 } else { 0.028 });
            assert_ne!(
                value(&before, &scene, id, "pan"),
                value(&doc, &scene, id, "pan")
            );
            assert_eq!(
                value(&before, &scene, id, "dimmer"),
                value(&doc, &scene, id, "dimmer")
            );
        }
        assert_eq!(
            raw(&before)["lighting"]["fixtures"],
            raw(&doc)["lighting"]["fixtures"]
        );
        assert_eq!(raw(&before)["stage"], raw(&doc)["stage"]);
    }
}
#[test]
fn flip_rejects_singular_member_and_unreachable_branch_as_one_transaction() {
    let (mut doc, scene, ids) = setup(true);
    position(
        &mut doc,
        &scene,
        &ids,
        json!({"op":"axes","panDegrees":"60","tiltDegrees":"40"}),
    )
    .unwrap();
    let mut root = raw(&doc);
    let m = doc.position_model(&ids[1]).unwrap().unwrap();
    let tilt = m
        .tilt
        .decode(value(&doc, &scene, &ids[1], "tilt"), true)
        .unwrap();
    root["lighting"]["fixtures"][1]["zeroCorrection"] =
        json!({"panDegrees":"0","tiltDegrees":(-tilt).to_string()});
    doc = decode(&root);
    let before = doc.clone();
    assert!(
        position(&mut doc, &scene, &ids, json!({"op":"flip"}))
            .unwrap_err()
            .contains("奇点")
    );
    assert_eq!(doc, before);
    root["lighting"]["fixtures"][1]
        .as_object_mut()
        .unwrap()
        .remove("zeroCorrection");
    root["lighting"]["profiles"][2]["positioning"]["tilt"]["minDegrees"] = json!("10");
    doc = decode(&root);
    let before = doc.clone();
    assert!(
        position(&mut doc, &scene, &ids, json!({"op":"flip"}))
            .unwrap_err()
            .contains("机械行程")
    );
    assert_eq!(doc, before);
    assert!(serde_json::from_value::<EditCommand>(json!({"op":"position","command":{"op":"flip","sceneId":scene,"fixtureIds":ids,"force":true}})).is_err());
}

#[test]
fn relative_edit_uses_scene_defaults_and_only_blocks_effects_on_requested_axes() {
    let (mut doc, scene, ids) = setup(true);
    let mut effect = motion(&ids);
    effect["channels"] = json!([effect["channels"][1].clone()]);
    put(&mut doc, &scene, &effect).unwrap();
    position(
        &mut doc,
        &scene,
        &ids,
        json!({"op":"offsetAxes","panDegrees":"1"}),
    )
    .unwrap();
    let before = doc.clone();
    assert!(
        position(
            &mut doc,
            &scene,
            &ids,
            json!({"op":"offsetAxes","tiltDegrees":"1"})
        )
        .is_err()
    );
    assert_eq!(doc, before);
    effect["enabled"] = json!(false);
    put(&mut doc, &scene, &effect).unwrap();
    // Remove the authored axis: relative adjustment now has an explicit default basis.
    for id in &ids {
        edit(&mut doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":id,"attribute":"tilt","mode":"remove","value":0})).unwrap();
    }
    position(
        &mut doc,
        &scene,
        &ids,
        json!({"op":"offsetAxes","tiltDegrees":"1"}),
    )
    .unwrap();
    for id in &ids {
        let m = doc.position_model(id).unwrap().unwrap();
        let actual = m
            .tilt
            .decode(value(&doc, &scene, id, "tilt"), true)
            .unwrap();
        let initial = m.tilt.decode(32768, true).unwrap();
        assert!((actual - initial - 1.0).abs() < 270.0 / 65535.0 / 2.0);
    }
}
