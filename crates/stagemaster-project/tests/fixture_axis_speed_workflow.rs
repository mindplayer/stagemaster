#[path = "support/fixture_axis_speed.rs"]
mod support;

use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, PackageSelection};
use support::{definition, edit, setup};

const SPEED: &str = "pan-tilt-speed";

fn set(doc: &mut Document, fixture: &str, scene: &str, value: u16) {
    edit(
        doc,
        json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,
        "attribute":SPEED,"mode":"literal","value":value}),
    )
    .unwrap();
}

#[test]
fn two_fixture_presets_update_and_release_use_original_sparse_semantics() {
    let (mut doc, fixtures, scenes) = setup(false);
    for fixture in &fixtures {
        set(&mut doc, fixture, &scenes[0], 10000);
    }
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"recordPreset","name":"速度控制位置",
        "sceneId":scenes[0],"fixtureIds":fixtures,"attributes":[SPEED]}}),
    )
    .unwrap();
    let preset = doc.view().presets[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"applyPreset","id":preset,
        "sceneId":scenes[1],"fixtureIds":fixtures,"attributes":[SPEED],"linked":true}}),
    )
    .unwrap();
    for fixture in &fixtures {
        set(&mut doc, fixture, &scenes[0], 50000);
    }
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"updatePreset","id":preset,
        "sceneId":scenes[0],"fixtureIds":fixtures,"attributes":[SPEED],"mode":"existing"}}),
    )
    .unwrap();
    let view = doc.view();
    assert_eq!(view.presets[0].values.len(), 2);
    assert!(view.presets[0].values.iter().all(|v| v.attribute == SPEED));
    let speeds = view.scenes[1]
        .values
        .iter()
        .filter(|v| v.attribute == SPEED)
        .collect::<Vec<_>>();
    assert_eq!(speeds.len(), 2);
    assert!(
        speeds
            .iter()
            .all(|v| v.value == Some(50000) && v.preset_id.as_deref() == Some(&preset))
    );
    assert!(
        view.scenes[1]
            .values
            .iter()
            .filter(|v| v.attribute != SPEED)
            .all(|v| v.preset_id.is_none())
    );
    for mode in ["release", "remove"] {
        edit(
            &mut doc,
            json!({"op":"setSceneValue","sceneId":scenes[1],"fixtureId":fixtures[0],
            "attribute":SPEED,"mode":mode,"value":0}),
        )
        .unwrap();
        let compiled = doc.compile_scene(&scenes[1]).unwrap();
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        assert_eq!(player.values()[3], 0x3456);
        assert_eq!(player.values()[7], 50000);
        assert_eq!(
            compiled.output.render(player.values()).unwrap().slots[26],
            0
        );
    }
    let before = doc.clone();
    assert!(
        edit(
            &mut doc,
            json!({"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":fixtures[0],
        "attribute":SPEED,"selection":{"functionKey":"reset","position":0}})
        )
        .is_err()
    );
    assert_eq!(doc, before);
}

#[test]
fn software_fade_and_portable_player_encode_same_values_without_new_clock() {
    let (mut doc, fixtures, scenes) = setup(true);
    for (scene, value) in scenes.iter().zip([10000, 50000]) {
        for fixture in &fixtures {
            set(&mut doc, fixture, scene, value);
        }
    }
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"速度位置渐变",
        "sceneId":scenes[0]}}),
    )
    .unwrap();
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let first = &mut raw["lighting"]["sequences"][0]["steps"][0];
    first["fade"] = json!({"ticks":"0","ticksPerSecond":"1000"});
    let mut second = first.clone();
    second["id"] = json!("aaaaaaaa-1111-4111-8111-000000000009");
    second["number"] = json!("2");
    second["sceneId"] = json!(scenes[1]);
    second["fade"] = json!({"ticks":"1000","ticksPerSecond":"1000"});
    raw["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let doc = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let id = doc.view().sequences[0].id.clone();
    let compiled = doc.compile_sequence(&id).unwrap();
    let built = doc
        .build_package(&[PackageSelection::Sequence { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    assert_eq!(archive.semantics(), 1);
    let portable = archive.load(built.bytes.as_slice(), 0).unwrap();
    assert_eq!(compiled.plan, portable.plan);
    let mut live = Player::new(compiled.plan, 0);
    let mut packaged = Player::new(portable.plan, 0);
    for player in [&mut live, &mut packaged] {
        player.execute(0, 0).unwrap();
        player.execute(1, 0).unwrap();
    }
    for now in (0..=1000).step_by(100) {
        live.advance(now).unwrap();
        packaged.advance(now).unwrap();
        let expected = u16::try_from(10000 + 40 * now).unwrap();
        assert_eq!(live.values()[3], expected);
        assert_eq!(live.values()[7], expected);
        let mut slots = [0; 512];
        portable
            .output
            .render(packaged.values(), &mut slots)
            .unwrap();
        assert_eq!(
            slots.as_slice(),
            compiled.output.render(live.values()).unwrap().slots
        );
        assert_eq!([slots[24], slots[21]], expected.to_be_bytes());
        assert_eq!([slots[35], slots[32]], expected.to_be_bytes());
        assert_eq!(slots[26], 0);
        assert_eq!(slots[37], 0);
    }
}

#[test]
fn exchange_preserves_both_fixture_values_and_rejects_partial_or_incompatible_batches() {
    let (mut doc, fixtures, scenes) = setup(false);
    set(&mut doc, &fixtures[0], &scenes[0], 0x1234);
    set(&mut doc, &fixtures[1], &scenes[0], 0xabcd);
    let original: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition(true)}}),
    )
    .unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    edit(&mut doc, json!({"op":"fixture","command":{"op":"exchange","fixtureIds":fixtures,"profileId":target,"layout":null}})).unwrap();
    let view = doc.view();
    assert!(view.fixtures.iter().all(|f| f.profile_id == target));
    assert_eq!(
        view.fixtures.iter().map(|f| f.address).collect::<Vec<_>>(),
        [Some(17), Some(28)]
    );
    let current: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    for key in ["scenes", "presets", "patches"] {
        assert_eq!(current["lighting"][key], original["lighting"][key]);
    }
    let compiled = doc.compile_scene(&scenes[0]).unwrap();
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let slots = compiled.output.render(player.values()).unwrap().slots;
    assert_eq!(
        [slots[24], slots[21], slots[35], slots[32]],
        [0x12, 0x34, 0xab, 0xcd]
    );
    let mut incompatible = definition(false);
    incompatible["channels"].as_array_mut().unwrap().pop();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":incompatible}}),
    )
    .unwrap();
    let missing_speed = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    for command in [
        json!({"op":"exchange","fixtureIds":fixtures,"profileId":missing_speed,"layout":null}),
        json!({"op":"exchange","fixtureIds":[fixtures[0],"ffffffff-ffff-4fff-8fff-ffffffffffff"],"profileId":target,"layout":null}),
        json!({"op":"exchange","fixtureIds":fixtures,"profileId":target,"layout":{"universe":1,"address":500,"gap":0}}),
        json!({"op":"saveProfile","id":target,"definition":definition(true)}),
    ] {
        assert!(edit(&mut doc, json!({"op":"fixture","command":command})).is_err());
        assert_eq!(doc, before);
    }
}
