#[path = "support/emitter_functions.rs"]
mod support;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stagemaster_playback::{OutputMaster, Player};
use stagemaster_project::{
    Document, FunctionSelection, ManualSceneReading, PackageSelection, ProfileDefault,
};
use support::{definition, edit, save, setup};
const PATTERN: &str = "emitter.pattern.shutter";
const WASH: &str = "emitter.wash.shutter";
const COLOR: &str = "emitter.pattern.color-wheel";
fn choose(
    doc: &mut Document,
    fixture: &str,
    scene: &str,
    attribute: &str,
    key: &str,
    position: u16,
) {
    edit(doc,json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":fixture,"attribute":attribute,"selection":{"functionKey":key,"position":position}})).unwrap();
}
#[test]
fn sparse_preset_and_manual_capture_keep_exact_unit_and_zero_ownership() {
    let (mut doc, ids, scene) = setup(definition());
    choose(&mut doc, &ids[0], &scene, PATTERN, "strobe", 32768);
    edit(&mut doc,json!({"op":"library","command":{"kind":"recordPreset","name":"仅图案光源快门","sceneId":scene,"fixtureIds":ids,"attributes":[PATTERN]}})).unwrap();
    let view = doc.view();
    assert_eq!(view.presets[0].values.len(), 2);
    assert!(
        view.presets[0]
            .values
            .iter()
            .all(|v| v.attribute == PATTERN)
    );
    let c = doc.compile_scene(&scene).unwrap();
    let open = ProfileDefault::Function(FunctionSelection {
        function_key: "open".into(),
        position: 0,
    });
    assert_eq!(
        c.output
            .manual_value(&ids[0], PATTERN, Some(&open))
            .unwrap()
            .1,
        Some(0)
    );
    assert_eq!(
        c.output.manual_value(&ids[0], PATTERN, None).unwrap().1,
        None
    );
    assert_eq!(
        c.output.manual_value(&ids[0], WASH, Some(&open)).unwrap().1,
        Some(0)
    );
    let layout = format!("{:x}", Sha256::digest(doc.encode().unwrap()));
    let capture = doc
        .capture_manual_scene(
            &layout,
            vec![ManualSceneReading {
                fixture_id: ids[0].clone(),
                attribute: PATTERN.into(),
                value: 0,
            }],
        )
        .unwrap();
    let id = doc.record_manual_scene(&capture, "开光仍持有").unwrap();
    let recorded = doc.view().scenes.into_iter().find(|s| s.id == id).unwrap();
    let value = recorded
        .values
        .iter()
        .find(|v| v.fixture_id == ids[0] && v.attribute == PATTERN)
        .unwrap();
    assert_eq!(value.function_value.as_ref().unwrap().function_key, "open");
    assert_eq!(value.value, Some(0));
    let raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let assignments = raw["lighting"]["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id)
        .unwrap()["assignments"]
        .as_array()
        .unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0]["target"]["attribute"], PATTERN);
    let layout = format!("{:x}", Sha256::digest(doc.encode().unwrap()));
    assert!(
        doc.capture_manual_scene(
            &layout,
            vec![ManualSceneReading {
                fixture_id: ids[0].clone(),
                attribute: COLOR.into(),
                value: 128 * 257
            }]
        )
        .is_err(),
        "自动换色空隙不能录入"
    );
}
#[test]
fn existing_player_and_portable_semantics_snap_functions_without_scaling_or_crossing_gaps() {
    let (mut doc, ids, first) = setup(definition());
    edit(&mut doc, json!({"op":"addScene","name":"受控切换"})).unwrap();
    let second = doc.view().scenes[1].id.clone();
    for (scene, key, dim) in [(&first, "open", 10000), (&second, "strobe", 50000)] {
        choose(
            &mut doc,
            &ids[0],
            scene,
            PATTERN,
            key,
            if key == "open" { 0 } else { 65535 },
        );
        edit(&mut doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":ids[0],"attribute":"dimmer","mode":"literal","value":dim})).unwrap();
    }
    choose(&mut doc, &ids[0], &second, COLOR, "slot-one", 0);
    choose(
        &mut doc,
        &ids[0],
        &second,
        "emitter.pattern.gobo-wheel",
        "shake-one",
        65535,
    );
    edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"切换","sceneId":first}}),
    )
    .unwrap();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let mut first = root["lighting"]["sequences"][0]["steps"][0].clone();
    first["fade"] = json!({"ticks":"0","ticksPerSecond":"1000"});
    let mut next = first.clone();
    next["id"] = json!("99999999-0000-4000-8000-000000000001");
    next["number"] = json!("2");
    next["sceneId"] = json!(second);
    next["delay"] = json!({"ticks":"50","ticksPerSecond":"1000"});
    next["fade"] = json!({"ticks":"100","ticksPerSecond":"1000"});
    root["lighting"]["sequences"][0]["steps"] = json!([first, next]);
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let id = doc.view().sequences[0].id.clone();
    let c = doc.compile_sequence(&id).unwrap();
    let built = doc
        .build_package(&[PackageSelection::Sequence { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    assert_eq!(archive.semantics(), 2);
    let portable = archive.load(built.bytes.as_slice(), 0).unwrap();
    assert_eq!(portable.plan, c.plan);
    let dim_index = c.output.manual_value(&ids[0], "dimmer", None).unwrap().0;
    let mut player = Player::new(c.plan, 0);
    player.execute(0, 0).unwrap();
    player.execute(1, 0).unwrap();
    let mut master = OutputMaster::default();
    master.set_percent(50).unwrap();
    for t in 0..=150 {
        player.advance(t).unwrap();
        let out = c
            .output
            .render_with_master(player.values(), master)
            .unwrap();
        assert_eq!(out.slots[23], if t < 50 { 0 } else { 255 });
        assert_eq!(out.slots[30], 0);
        assert_eq!(out.slots[24], if t < 50 { 0 } else { 16 });
        assert_eq!(out.slots[25], if t < 50 { 0 } else { 127 });
        if t == 100 {
            assert_eq!(player.values()[dim_index], 30000);
        }
        let mut slots = [0; 512];
        portable.output.render(player.values(), &mut slots).unwrap();
        assert_eq!(
            slots.as_slice(),
            c.output.render(player.values()).unwrap().slots
        );
    }
}
#[test]
fn explicit_color_slot_remap_is_unit_exact_and_batch_rejections_are_atomic() {
    let (mut doc, ids, scene) = setup(definition());
    for id in &ids {
        choose(&mut doc, id, &scene, COLOR, "slot-one", 0);
    }
    let mut def = definition();
    def["channels"][8]["functions"][1]["dmxDefault"] = json!(20);
    def["channels"][8]["functions"][1]["appearance"] = json!({"kind":"color","colors":["#FF0000"]});
    save(&mut doc, def).unwrap();
    let target = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    assert!(edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":ids,"profileId":target,"layout":null}})).is_err());
    assert_eq!(doc, before);
    edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":ids,"profileId":target,"layout":null,"allowColorSlotRemap":true}})).unwrap();
    let c = doc.compile_scene(&scene).unwrap();
    let mut player = Player::new(c.plan, 0);
    player.execute(0, 0).unwrap();
    let slots = c.output.render(player.values()).unwrap().slots;
    assert_eq!([slots[24], slots[42]], [20, 20]);
    let mut def = definition();
    def["channels"][6]["functions"][1]["dmxDefault"] = json!(32);
    save(&mut doc, def).unwrap();
    let bad = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    assert!(edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":ids,"profileId":bad,"layout":null,"allowColorSlotRemap":true}})).is_err());
    assert_eq!(doc, before);
}
