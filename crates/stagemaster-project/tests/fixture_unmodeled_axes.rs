use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{Document, EditCommand, ProfileFile};

fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(command).unwrap())
}
// Tests the confirmed mapping subset; not a complete or field-validated 11-channel mode.
fn definition() -> Value {
    let mut colors = (0..14)
        .map(|i| {
            json!({
                "key":format!("slot-{}", i+1),"name":format!("颜色档位 {}",i+1),
                "mode":"slot","dmxFrom":i*10,"dmxTo":i*10+9,"dmxDefault":i*10+4
            })
        })
        .collect::<Vec<_>>();
    colors.push(json!({"key":"automatic","name":"自动换色","mode":"range","dmxFrom":140,"dmxTo":255,"dmxDefault":140}));
    json!({"name":"十一通道映射边界测试","manufacturer":"资料未署名","model":"30W 图案灯","mode":"11CH 测试子集","footprint":11,"channels":[
        {"attribute":"pan","coarse":1,"fine":2,"defaultValue":32768},
        {"attribute":"tilt","coarse":3,"fine":4,"defaultValue":32768},
        {"attribute":"color-wheel","coarse":5,"fine":null,"defaultValue":{"functionKey":"slot-1","position":0},"functions":colors},
        {"attribute":"dimmer","coarse":8,"fine":null,"defaultValue":0}]})
}
fn setup() -> Document {
    let mut doc = Document::new("未知几何").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition()}}),
    )
    .unwrap();
    let v = doc.view();
    edit(&mut doc,json!({"op":"addFixture","name":"实测灯","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":9})).unwrap();
    edit(&mut doc, json!({"op":"addScene","name":"静态通道"})).unwrap();
    doc
}

#[test]
fn raw_axes_and_color_slots_compile_exact_bytes_without_inventing_a_model() {
    let mut doc = setup();
    let v = doc.view();
    let f = &v.fixtures[0].id;
    let s = &v.scenes[0].id;
    assert!(v.profiles.last().unwrap().authorable);
    assert!(doc.position_model(f).unwrap().is_none());
    for (axis, value) in [("pan", 0x1234), ("tilt", 0xabcd)] {
        edit(&mut doc,json!({"op":"setSceneValue","sceneId":s,"fixtureId":f,"attribute":axis,"mode":"literal","value":value})).unwrap();
    }
    for (key, position, expected) in (1..=14).map(|i| (format!("slot-{i}"), 0, (i - 1) * 10 + 4)) {
        edit(&mut doc,json!({"op":"setSceneFunctionValue","sceneId":s,"fixtureId":f,"attribute":"color-wheel","selection":{"functionKey":key,"position":position}})).unwrap();
        let compiled = doc.compile_scene(s).unwrap();
        let mut player = Player::new(compiled.plan, 0);
        player.execute(0, 0).unwrap();
        let frame = compiled.output.render(player.values()).unwrap();
        assert_eq!(&frame.slots[8..12], &[0x12, 0x34, 0xab, 0xcd]);
        assert_eq!(u32::from(frame.slots[12]), expected);
        assert_eq!(&frame.slots[13..19], &[0; 6]);
    }
    let before = doc.clone();
    for position in [0, 65535] {
        assert!(edit(&mut doc,json!({"op":"setSceneFunctionValue","sceneId":s,"fixtureId":f,"attribute":"color-wheel","selection":{"functionKey":"automatic","position":position}})).unwrap_err().contains("已屏蔽"));
        assert_eq!(doc, before);
    }
}

#[test]
fn missing_geometry_survives_project_and_portable_file_roundtrips() {
    let doc = setup();
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    let profile = doc.view().profiles.pop().unwrap();
    let file = doc.profile_file(&profile.id).unwrap().encode().unwrap();
    let decoded = ProfileFile::decode(&file).unwrap();
    assert!(decoded.definition().positioning.is_none());
    assert_eq!(decoded.encode().unwrap(), file);
    assert_eq!(
        serde_json::to_value(decoded.definition()).unwrap()["channels"],
        definition()["channels"]
    );
}

#[test]
fn all_physical_operations_refuse_missing_model_without_mutating_the_project() {
    let mut doc = setup();
    let v = doc.view();
    let mut commands = vec![
        json!({"op":"axes","panDegrees":"10","tiltDegrees":null}),
        json!({"op":"offsetAxes","panDegrees":"2","tiltDegrees":null}),
        json!({"op":"flip"}),
        json!({"op":"home"}),
        json!({"op":"aim","targetMeters":{"x":"0","y":"0","z":"0"},"branch":null}),
    ];
    for c in &mut commands {
        c["sceneId"] = json!(v.scenes[0].id);
        c["fixtureIds"] = json!([v.fixtures[0].id]);
    }
    commands.push(json!({"op":"calibrate","fixtureId":v.fixtures[0].id,"correction":{"panDegrees":"1","tiltDegrees":"0"}}));
    let before = doc.clone();
    for c in commands {
        assert!(edit(&mut doc, json!({"op":"position","command":c})).is_err());
        assert_eq!(doc, before);
    }
}

#[test]
fn partial_axes_and_model_without_axes_remain_invalid() {
    let mut doc = Document::new("边界").unwrap();
    let before = doc.clone();
    let mut partial = definition();
    partial["channels"].as_array_mut().unwrap().remove(1);
    assert!(
        edit(
            &mut doc,
            json!({"op":"fixture","command":{"op":"saveProfile","definition":partial}})
        )
        .is_err()
    );
    assert_eq!(doc, before);
    let mut fixed = definition();
    fixed["channels"].as_array_mut().unwrap().drain(..2);
    fixed["positioning"] = json!({"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":false},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}});
    assert!(
        edit(
            &mut doc,
            json!({"op":"fixture","command":{"op":"saveProfile","definition":fixed}})
        )
        .is_err()
    );
    assert_eq!(doc, before);
}
