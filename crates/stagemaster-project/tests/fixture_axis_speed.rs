#[path = "support/fixture_axis_speed.rs"]
mod support;

use serde_json::json;
use stagemaster_playback::OutputMaster;
use stagemaster_project::{Document, ProfileFile};
use support::{definition, edit, setup};

#[test]
fn combined_axis_speed_is_authorable_with_exact_nonzero_address_defaults() {
    let (doc, _, scenes) = setup(false);
    let view = doc.view();
    assert!(view.profiles.last().unwrap().authorable);
    assert_eq!(view.fixtures[0].attributes[3].label, "两轴速度控制");
    let compiled = doc.compile_scene(&scenes[0]).unwrap();
    let slots = compiled
        .output
        .render(compiled.plan.defaults())
        .unwrap()
        .slots;
    assert_eq!(
        &slots[16..27],
        &[0x12, 0x34, 0xab, 0xcd, 0, 0, 0, 255, 0x34, 0, 0]
    );
    assert_eq!(&slots[27..38], &slots[16..27]);
    assert!(slots[..16].iter().chain(&slots[38..]).all(|v| *v == 0));
    assert!(compiled.plan.snap_attributes().is_empty());
}

#[test]
fn speed_endpoints_and_fine_before_coarse_are_not_intensity_or_position() {
    for fine in [false, true] {
        let (mut doc, fixtures, scenes) = setup(fine);
        for value in [0, 1, 0x1234, 0xffff] {
            edit(
                &mut doc,
                json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixtures[0],
                "attribute":"pan-tilt-speed","mode":"literal","value":value}),
            )
            .unwrap();
            let compiled = doc.compile_scene(&scenes[0]).unwrap();
            let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
            player.execute(0, 0).unwrap();
            let mut master = OutputMaster::default();
            master.set_percent(37).unwrap();
            master.set_blackout(true);
            let frame = compiled
                .output
                .render_with_master(player.values(), master)
                .unwrap();
            assert_eq!(&frame.slots[16..20], &[0x12, 0x34, 0xab, 0xcd]);
            assert_eq!(frame.slots[23], 0);
            let value = u16::try_from(value).unwrap();
            let expected = if fine {
                value.to_be_bytes()[0]
            } else {
                u8::try_from((u32::from(value) + 128) / 257).unwrap()
            };
            assert_eq!(frame.slots[24], expected);
            assert_eq!(
                frame.slots[21],
                if fine { value.to_be_bytes()[1] } else { 0 }
            );
            assert_eq!(frame.slots[26], 0, "复位通道不由速度控制");
        }
    }
}

#[test]
fn profile_and_project_roundtrips_keep_mapping_and_ltp_without_new_format() {
    let (doc, _, _) = setup(true);
    let bytes = doc.encode().unwrap();
    assert_eq!(doc, Document::decode(&bytes).unwrap());
    let profile = doc.view().profiles.pop().unwrap();
    let file = doc.profile_file(&profile.id).unwrap().encode().unwrap();
    let decoded = ProfileFile::decode(&file).unwrap();
    assert_eq!(decoded.encode().unwrap(), file);
    assert_eq!(
        serde_json::to_value(decoded.definition()).unwrap(),
        definition(true)
    );
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        raw["requires"],
        json!([{"key":"lighting.basic","version":1}])
    );
    assert_eq!(
        raw["lighting"]["profiles"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["attributes"][3]["mix"],
        "ltp"
    );
}

#[test]
fn invalid_speed_definitions_and_partial_axes_fail_atomically() {
    let (mut doc, _, _) = setup(false);
    let before = doc.clone();
    for (path, value) in [
        ("/channels/3/coarse", json!(8)),
        ("/channels/3/coarse", json!(12)),
        ("/channels/3/fine", json!(9)),
        ("/channels/3/attribute", json!("pan")),
        ("/channels/3/attribute", json!("reset")),
        (
            "/channels/3/defaultValue",
            json!({"functionKey":"fast","position":0}),
        ),
        ("/channels/3/defaultValue", json!(65536)),
        ("/channels/3/defaultValue", json!(-1)),
    ] {
        let mut def = definition(false);
        *def.pointer_mut(path).unwrap() = value;
        assert!(
            edit(
                &mut doc,
                json!({"op":"fixture","command":{"op":"saveProfile","definition":def}})
            )
            .is_err()
        );
        assert_eq!(doc, before);
    }
    for axes in [0, 1] {
        let mut def = definition(false);
        def["channels"].as_array_mut().unwrap().drain(..2 - axes);
        assert!(
            edit(
                &mut doc,
                json!({"op":"fixture","command":{"op":"saveProfile","definition":def}})
            )
            .is_err()
        );
        assert_eq!(doc, before);
    }
    let mut def = definition(false);
    def["channels"][3]["functions"] = json!([]);
    assert!(
        edit(
            &mut doc,
            json!({"op":"fixture","command":{"op":"saveProfile","definition":def}})
        )
        .unwrap_err()
        .contains("线性")
    );
    assert_eq!(doc, before);
}
