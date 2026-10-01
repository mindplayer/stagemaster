use serde_json::{Value, json};
use stagemaster_playback::{OutputMaster, Player};
use stagemaster_project::{Document, EditCommand, PackageSelection, ProfileFile};
fn edit(d: &mut Document, c: Value) -> Result<(), String> {
    d.edit(serde_json::from_value::<EditCommand>(c).unwrap())
}
fn definition() -> Value {
    json!({"name":"镜头模式","manufacturer":"测试","model":"镜头","mode":"六通道","footprint":6,
      "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":65535},
      {"attribute":"zoom","coarse":5,"fine":2,"defaultValue":4660},
      {"attribute":"focus","coarse":3,"fine":null,"defaultValue":32768},
      {"attribute":"iris","coarse":4,"fine":null,"defaultValue":65535}]})
}
fn setup() -> (Document, String, Vec<String>) {
    let mut d = Document::new("镜头控制").unwrap();
    edit(
        &mut d,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition()}}),
    )
    .unwrap();
    let v = d.view();
    edit(&mut d, json!({"op":"addFixture","name":"镜头灯","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    for name in ["甲", "乙"] {
        edit(&mut d, json!({"op":"addScene","name":name})).unwrap();
    }
    let v = d.view();
    (
        d,
        v.fixtures[0].id.clone(),
        v.scenes.iter().map(|s| s.id.clone()).collect(),
    )
}
#[test]
fn control_positions_roundtrip_and_do_not_depend_on_intensity_master() {
    let (d, _, scenes) = setup();
    let profile = d.view().profiles.pop().unwrap();
    assert!(profile.authorable);
    assert_eq!(
        d.view().fixtures[0]
            .attributes
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        ["亮度", "变焦", "调焦", "光圈"]
    );
    let before = d.encode().unwrap();
    let file = d.profile_file(&profile.id).unwrap();
    let imported = ProfileFile::decode(&file.encode().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(imported.definition()).unwrap(),
        serde_json::to_value(file.definition()).unwrap()
    );
    let c = d.compile_scene(&scenes[0]).unwrap();
    assert!(c.plan.snap_attributes().is_empty());
    assert_eq!(
        &c.output.render(c.plan.defaults()).unwrap().slots[..6],
        &[255, 0x34, 128, 255, 0x12, 0]
    );
    let mut master = OutputMaster::default();
    master.set_blackout(true);
    assert_eq!(
        &c.output
            .render_with_master(c.plan.defaults(), master)
            .unwrap()
            .slots[..6],
        &[0, 0x34, 128, 255, 0x12, 0]
    );
    assert_eq!(d, Document::decode(&before).unwrap());
    assert_eq!(before, d.encode().unwrap());
}
#[test]
fn presets_fades_and_portable_player_preserve_lens_control_positions() {
    let (mut d, fixture, scenes) = setup();
    let attrs = ["zoom", "focus", "iris"];
    for a in attrs {
        edit(&mut d, json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":a,"mode":"literal","value":10000})).unwrap();
    }
    edit(&mut d,json!({"op":"library","command":{"kind":"recordPreset","name":"镜头预设","sceneId":scenes[0],"fixtureIds":[fixture],"attributes":attrs}})).unwrap();
    let preset = d.view().presets[0].id.clone();
    edit(&mut d,json!({"op":"library","command":{"kind":"applyPreset","id":preset,"sceneId":scenes[1],"fixtureIds":[fixture],"attributes":attrs,"linked":true}})).unwrap();
    assert_eq!(
        d.view().scenes[1]
            .values
            .iter()
            .filter(|v| attrs.contains(&v.attribute.as_str()))
            .map(|v| v.value)
            .collect::<Vec<_>>(),
        vec![Some(10000); 3]
    );
    for a in attrs {
        edit(&mut d, json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":a,"mode":"literal","value":50000})).unwrap();
    }
    edit(&mut d,json!({"op":"library","command":{"kind":"updatePreset","id":preset,"sceneId":scenes[0],"fixtureIds":[fixture],"attributes":attrs,"mode":"existing"}})).unwrap();
    assert_eq!(
        d.view().scenes[1]
            .values
            .iter()
            .filter(|v| attrs.contains(&v.attribute.as_str()))
            .map(|v| v.value)
            .collect::<Vec<_>>(),
        vec![Some(50000); 3]
    );
    for a in attrs {
        edit(&mut d, json!({"op":"setSceneValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":a,"mode":"literal","value":10000})).unwrap();
    }
    edit(
        &mut d,
        json!({"op":"sequence","command":{"kind":"add","name":"镜头渐变","sceneId":scenes[0]}}),
    )
    .unwrap();
    let mut raw: Value = serde_json::from_slice(&d.encode().unwrap()).unwrap();
    let first = &mut raw["lighting"]["sequences"][0]["steps"][0];
    first["fade"] = json!({"ticks":"0","ticksPerSecond":"1000"});
    let mut second = first.clone();
    second["id"] = json!("99999999-1111-4111-8111-000000000080");
    second["number"] = json!("2");
    second["sceneId"] = json!(scenes[1]);
    second["fade"] = json!({"ticks":"1000","ticksPerSecond":"1000"});
    raw["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let mut d = Document::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let id = d.view().sequences[0].id.clone();
    let c = d.compile_sequence(&id).unwrap();
    let built = d
        .build_package(&[PackageSelection::Sequence { id }])
        .unwrap();
    let archive = stagemaster_package::Archive::open(built.bytes.as_slice()).unwrap();
    assert_eq!(archive.semantics(), 1);
    let program = archive.load(built.bytes.as_slice(), 0).unwrap();
    assert_eq!(c.plan, program.plan);
    let mut live = Player::new(c.plan, 0);
    let mut portable = Player::new(program.plan, 0);
    for p in [&mut live, &mut portable] {
        p.execute(0, 0).unwrap();
        p.execute(1, 0).unwrap();
    }
    for t in (0..=1000).step_by(100) {
        live.advance(t).unwrap();
        portable.advance(t).unwrap();
        let expected = u16::try_from(10000 + 40 * t).unwrap();
        assert_eq!(&live.values()[1..], &[expected; 3]);
        let mut slots = [0; 512];
        program
            .output
            .render(portable.values(), &mut slots)
            .unwrap();
        assert_eq!(
            slots.as_slice(),
            c.output.render(live.values()).unwrap().slots
        );
    }
    edit(&mut d,json!({"op":"setSceneValue","sceneId":scenes[1],"fixtureId":fixture,"attribute":"zoom","mode":"release","value":0})).unwrap();
    let c = d.compile_scene(&scenes[1]).unwrap();
    let mut p = Player::new(c.plan, 0);
    p.execute(0, 0).unwrap();
    assert_eq!(p.values()[1], 4660);
}
#[test]
fn lens_mode_rejects_duplicate_channels_functions_and_unknown_controls_atomically() {
    let (mut d, _, _) = setup();
    let before = d.clone();
    for (path, value) in [
        ("/channels/1/coarse", json!(1)),
        ("/channels/1/attribute", json!("focus")),
        ("/channels/1/attribute", json!("reset")),
        (
            "/channels/1/defaultValue",
            json!({"functionKey":"open","position":0}),
        ),
    ] {
        let mut def = definition();
        *def.pointer_mut(path).unwrap() = value;
        assert!(
            edit(
                &mut d,
                json!({"op":"fixture","command":{"op":"saveProfile","definition":def}})
            )
            .is_err()
        );
        assert_eq!(d, before);
    }
    let mut def = definition();
    def["channels"][1]["functions"] = json!([{ "key":"zoom", "name":"变焦", "mode":"range", "dmxFrom":0,"dmxTo":65535,"dmxDefault":0 }]);
    assert!(
        edit(
            &mut d,
            json!({"op":"fixture","command":{"op":"saveProfile","definition":def}})
        )
        .unwrap_err()
        .contains("线性")
    );
    assert_eq!(d, before);
}
