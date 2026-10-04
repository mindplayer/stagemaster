#[path = "support/fixture_emitters.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_engine::live::{Frame, Kind, LiveMixer, Source};
use stagemaster_playback::{OutputMaster, Player};
use stagemaster_project::{Document, PackageSelection};
use support::{definition, edit, save, setup};
const WHITE: &str = "emitter.wash.white";
#[test]
fn live_mixer_keeps_manual_ownership_while_master_attenuates_once() {
    for global in [false, true] {
        let mut def = definition();
        if !global {
            def["channels"].as_array_mut().unwrap().remove(0);
        }
        let (doc, _, scene) = setup(def);
        let player = doc.compile_live_scene(&scene, 0).unwrap();
        let mut output = player.prepare_output().unwrap();
        let mut mixer = LiveMixer::new([1; 16], player.layout().clone(), 1).unwrap();
        let layout = mixer.layout().id();
        let source = mixer
            .open(
                Source {
                    id: [2; 16],
                    kind: Kind::Programmer,
                },
                0,
                layout,
            )
            .unwrap();
        let len = mixer.layout().attributes().len();
        mixer
            .publish(
                source,
                Frame {
                    layout,
                    serial: 1,
                    values: &vec![Some(65535); len],
                    assert: &vec![false; len],
                },
            )
            .unwrap();
        let mut master = OutputMaster::default();
        master.set_percent(50).unwrap();
        let mut slots = [0; 512];
        output
            .render_with_master(&mixer, &mut slots, master)
            .unwrap();
        assert_eq!(slots[22], if global { 255 } else { 128 });
        assert_eq!(slots[29], if global { 255 } else { 128 });
        if global {
            assert_eq!(slots[21], 128);
        }
        assert!(
            output
                .winners()
                .iter()
                .all(|winner| *winner == Some(source))
        );
        master.set_blackout(true);
        output
            .render_with_master(&mixer, &mut slots, master)
            .unwrap();
        assert_eq!(slots[22], if global { 255 } else { 0 });
        assert_eq!(slots[29], if global { 255 } else { 0 });
        assert!(
            output
                .winners()
                .iter()
                .all(|winner| *winner == Some(source))
        );
    }
}
fn set(doc: &mut Document, fixture: &str, scene: &str, key: &str, value: u16) {
    edit(doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"mode":"literal","value":value})).unwrap();
}
#[test]
fn white_fine_before_coarse_endpoints_and_optional_local_dimmer_masks() {
    let mut def = definition();
    def["channels"][5]["fine"] = json!(4);
    let (mut doc, ids, scene) = setup(def);
    for value in [0, 1, 0x1234, 65535] {
        set(&mut doc, &ids[0], &scene, WHITE, value);
        let compiled = doc.compile_scene(&scene).unwrap();
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        let slots = compiled.output.render(player.values()).unwrap().slots;
        assert_eq!([slots[29], slots[19]], value.to_be_bytes());
        assert_eq!(&slots[30..34], &[0, 0, 0, 0]);
    }
    let mut def = definition();
    def["channels"].as_array_mut().unwrap().remove(0);
    def["channels"][4]["defaultValue"] = json!(65535);
    def["channels"].as_array_mut().unwrap().push(
        json!({"attribute":"emitter.wash.dimmer","coarse":15,"fine":null,"defaultValue":65535}),
    );
    let (doc, _, scene) = setup(def);
    let compiled = doc.compile_scene(&scene).unwrap();
    let mut master = OutputMaster::default();
    master.set_percent(50).unwrap();
    let slots = compiled
        .output
        .render_with_master(compiled.plan.defaults(), master)
        .unwrap()
        .slots;
    assert_eq!(slots[30], 128);
    assert_eq!(slots[26], 255);
    assert_eq!(slots[29], 255);
}
#[test]
fn sparse_presets_keep_independent_targets_and_zero_is_not_release() {
    let (mut doc, ids, scene) = setup(definition());
    set(&mut doc, &ids[0], &scene, "emitter.pattern.dimmer", 0);
    set(&mut doc, &ids[0], &scene, WHITE, 50000);
    edit(&mut doc,json!({"op":"library","command":{"kind":"recordPreset","name":"仅白光","sceneId":scene,"fixtureIds":ids,"attributes":[WHITE]}})).unwrap();
    let view = doc.view();
    assert_eq!(view.presets[0].values.len(), 2);
    assert!(view.presets[0].values.iter().all(|v| v.attribute == WHITE));
    assert_eq!(
        view.presets[0]
            .values
            .iter()
            .find(|v| v.fixture_id == ids[0])
            .unwrap()
            .value,
        Some(50000)
    );
    let c = doc.compile_scene(&scene).unwrap();
    assert_eq!(
        c.output
            .manual_value(
                &ids[0],
                WHITE,
                Some(&stagemaster_project::ProfileDefault::Normalized(0))
            )
            .unwrap()
            .1,
        Some(0)
    );
    assert_eq!(c.output.manual_value(&ids[0], WHITE, None).unwrap().1, None);
    assert!(
        c.output
            .manual_value(
                &ids[0],
                "white",
                Some(&stagemaster_project::ProfileDefault::Normalized(65535))
            )
            .is_err()
    );
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    assert_eq!(c.output.render(p.values()).unwrap().slots[22], 0);
    edit(&mut doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":ids[0],"attribute":"emitter.pattern.dimmer","mode":"release","value":0})).unwrap();
    let c = doc.compile_scene(&scene).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    assert_eq!(c.output.render(p.values()).unwrap().slots[22], 128);
    assert_eq!(
        doc.view().scenes[0]
            .values
            .iter()
            .find(|v| v.attribute == WHITE)
            .unwrap()
            .value,
        Some(50000)
    );
}
#[test]
fn portable_sequence_uses_original_fade_and_exact_white_mapping() {
    let mut def = definition();
    def["channels"][5]["fine"] = json!(4);
    let (mut doc, ids, first) = setup(def);
    edit(&mut doc, json!({"op":"addScene","name":"白光目标"})).unwrap();
    let second = doc.view().scenes[1].id.clone();
    set(&mut doc, &ids[0], &first, WHITE, 10000);
    set(&mut doc, &ids[0], &second, WHITE, 50000);
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"独立白光渐变","sceneId":first}}),
    )
    .unwrap();
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    raw["lighting"]["sequences"][0]["steps"][0]["fade"] =
        json!({"ticks":"0","ticksPerSecond":"1000"});
    let mut next = raw["lighting"]["sequences"][0]["steps"][0].clone();
    next["id"] = json!("bbbbbbbb-2222-4222-8222-000000000011");
    next["number"] = json!("2");
    next["sceneId"] = json!(second);
    next["fade"] = json!({"ticks":"1000","ticksPerSecond":"1000"});
    raw["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(next);
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
    let mut player = Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    player.execute(1, 0).unwrap();
    for now in (0..=1000).step_by(100) {
        player.advance(now).unwrap();
        let expected = u16::try_from(10000 + 40 * now).unwrap();
        let mut slots = [0; 512];
        portable.output.render(player.values(), &mut slots).unwrap();
        assert_eq!([slots[29], slots[19]], expected.to_be_bytes());
        assert_eq!(
            slots.as_slice(),
            compiled.output.render(player.values()).unwrap().slots
        );
    }
}
#[test]
fn batch_exchange_requires_exact_unit_identity_and_preserves_scene_values() {
    let (mut doc, ids, scene) = setup(definition());
    set(&mut doc, &ids[0], &scene, WHITE, 0x1234);
    let mut def = definition();
    def["channels"][5]["fine"] = json!(4);
    def["emitters"][1]["name"] = json!("染色光源改名");
    save(&mut doc, def).unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":ids,"profileId":target,"layout":null}})).unwrap();
    assert!(doc.view().fixtures.iter().all(|f| f.profile_id == target));
    assert_eq!(
        doc.view().scenes[0]
            .values
            .iter()
            .find(|v| v.fixture_id == ids[0] && v.attribute == WHITE)
            .unwrap()
            .value,
        Some(0x1234)
    );
    let mut def = definition();
    def["emitters"][0]["key"] = json!("other");
    def["channels"][1]["attribute"] = json!("emitter.other.dimmer");
    save(&mut doc, def).unwrap();
    let wrong = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    for command in [
        json!({"op":"exchange","fixtureIds":ids,"profileId":wrong,"layout":null}),
        json!({"op":"exchange","fixtureIds":ids,"profileId":target,"layout":{"universe":1,"address":500,"gap":0}}),
    ] {
        assert!(edit(&mut doc, json!({"op":"fixture","command":command})).is_err());
        assert_eq!(doc, before);
    }
}
