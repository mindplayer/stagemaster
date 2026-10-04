use serde_json::json;
use stagemaster_engine::live::{Frame, Kind, LiveMixer, Source};
use stagemaster_playback::OutputMaster;
use stagemaster_project::Document;
fn setup(keys: &[&str]) -> (Document, String) {
    let mut doc = Document::new("总控验收").unwrap();
    let mut authorable = keys.to_vec();
    if keys == ["pan", "tilt"] {
        authorable.push("dimmer");
    }
    if keys == ["red", "green"] {
        authorable.push("blue");
    }
    doc.edit(serde_json::from_value(json!({"op":"fixture","command":{"op":"saveProfile","definition":{
        "name":"验收档案","manufacturer":"测试","model":"总控","mode":"自定义","footprint":authorable.len(),
        "positioning": if keys.contains(&"pan") { json!({"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":false},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}}) } else { serde_json::Value::Null },
        "channels":authorable.iter().enumerate().map(|(i,k)|json!({"attribute":k,"coarse":i+1,"fine":null,"defaultValue":65535})).collect::<Vec<_>>()
    }}})).unwrap()).unwrap();
    // Importable numeric profiles may intentionally exceed the narrower UI authoring set.
    let mut root: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let profile = root["lighting"]["profiles"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    profile["attributes"]
        .as_array_mut()
        .unwrap()
        .retain(|a| keys.contains(&a["key"].as_str().unwrap()));
    profile["channels"]
        .as_array_mut()
        .unwrap()
        .retain(|c| keys.contains(&c["attribute"].as_str().unwrap()));
    for (index, channel) in profile["channels"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        channel["offsets"] = json!([index]);
    }
    profile["footprint"] = json!(keys.len());
    doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let view = doc.view();
    doc.edit(serde_json::from_value(json!({"op":"addFixture","name":"灯","profileId":view.profiles.last().unwrap().id,"domainId":view.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
    doc.edit(serde_json::from_value(json!({"op":"addScene","name":"默认全值"})).unwrap())
        .unwrap();
    let id = doc.view().scenes[0].id.clone();
    (doc, id)
}
#[test]
fn live_output_uses_the_same_conservative_intensity_mask_once_after_mixing() {
    for (keys, scaled) in [
        (
            vec!["dimmer", "red", "green", "blue", "pan", "tilt"],
            vec!["dimmer"],
        ),
        (
            vec!["red", "green", "blue", "pan", "tilt"],
            vec!["red", "green", "blue"],
        ),
        (vec!["pan", "tilt"], vec![]),
        (vec!["red", "green"], vec![]),
    ] {
        let (doc, scene) = setup(&keys);
        let compiled = doc.compile_scene(&scene).unwrap();
        let indices: Vec<_> = scaled
            .iter()
            .map(|key| {
                compiled
                    .output
                    .manual_value(&doc.view().fixtures[0].id, key, None)
                    .unwrap()
                    .0
            })
            .collect();
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
        mixer
            .publish(
                source,
                Frame {
                    layout,
                    serial: 1,
                    values: &vec![Some(65535); keys.len()],
                    assert: &vec![false; keys.len()],
                },
            )
            .unwrap();
        let mut slots = [0; 512];
        let mut master = OutputMaster::default();
        for (percent, black, expected) in [
            (50, false, 32768),
            (37, true, 0),
            (37, false, 24248),
            (100, false, 65535),
        ] {
            master.set_percent(percent).unwrap();
            master.set_blackout(black);
            output
                .render_with_master(&mixer, &mut slots, master)
                .unwrap();
            for (i, attribute) in mixer.layout().attributes().iter().enumerate() {
                let encoded = if indices.contains(&i) {
                    expected
                } else {
                    65535
                };
                assert_eq!(output.values()[i], encoded, "{keys:?} {attribute:?}");
                assert_eq!(output.winners()[i], Some(source));
            }
        }
        output.render(&mixer, &mut slots).unwrap();
        assert!(output.values().iter().all(|v| *v == 65535));
    }
}

#[test]
fn master_changes_one_intensity_path_before_encoding_and_never_mutates_plan() {
    for (keys, scaled, unsupported) in [
        (
            vec!["dimmer", "red", "green", "blue", "pan", "tilt"],
            vec![0],
            0,
        ),
        (
            vec!["red", "green", "blue", "pan", "tilt"],
            vec![0, 1, 2],
            0,
        ),
        (vec!["pan", "tilt"], vec![], 1),
        (vec!["red", "green"], vec![], 1),
    ] {
        let (doc, id) = setup(&keys);
        assert_eq!(doc.uncontrolled_intensity_fixtures(), unsupported);
        let compiled = doc.compile_scene(&id).unwrap();
        let values = vec![65535; keys.len()];
        let before = serde_json::to_value(compiled.output.render(&values).unwrap()).unwrap();
        let mut master = OutputMaster::default();
        for (percent, blackout, expected) in
            [(50, false, 32768), (50, true, 0), (100, false, 65535)]
        {
            master.set_percent(percent).unwrap();
            master.set_blackout(blackout);
            let frame = compiled.output.render_with_master(&values, master).unwrap();
            for (i, attr) in frame.fixtures[0].attributes.iter().enumerate() {
                let value = if scaled.contains(&i) { expected } else { 65535 };
                assert_eq!(attr.value, value, "{}", keys[i]);
                assert_eq!(frame.slots[i], u8::try_from(value >> 8).unwrap());
            }
        }
        assert_eq!(
            serde_json::to_value(compiled.output.render(&values).unwrap()).unwrap(),
            before
        );
        assert_eq!(values, vec![65535; keys.len()]);
        assert!(compiled.output.render_with_master(&[], master).is_err());
    }
}

#[test]
fn intensity_is_scaled_before_coarse_fine_encoding() {
    let (doc, scene) = setup(&["dimmer", "red", "green", "blue"]);
    let mut root: serde_json::Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let profile = root["lighting"]["profiles"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    profile["footprint"] = json!(5);
    profile["channels"][0]["encoding"] = json!("u16-be");
    profile["channels"][0]["offsets"] = json!([0, 1]);
    for index in 1..4 {
        profile["channels"][index]["offsets"] = json!([index + 1]);
    }
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let output = doc.compile_scene(&scene).unwrap().output;
    let mut master = OutputMaster::default();
    master.set_percent(50).unwrap();
    let frame = output.render_with_master(&[65535; 4], master).unwrap();
    assert_eq!(&frame.slots[..5], &[128, 0, 255, 255, 255]);
    assert_eq!(frame.fixtures[0].attributes[0].value, 32768);
}
