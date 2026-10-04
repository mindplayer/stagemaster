use serde_json::json;
use stagemaster_project::{Document, EditCommand};

fn edit(doc: &mut Document, value: serde_json::Value) {
    doc.edit(serde_json::from_value::<EditCommand>(value).unwrap())
        .unwrap();
}
#[test]
fn internal_program_keeps_body_and_output_but_never_fabricates_trajectory_or_light() {
    let mut doc = Document::new("内置程序预演边界").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":{
        "name":"内置程序软件边界","manufacturer":"契约","model":"非实灯","mode":"双通道","footprint":2,
        "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":65535},
        {"attribute":"fixture-program","coarse":2,"fine":null,"defaultValue":{"functionKey":"external","position":0},"functions":[
        {"key":"external","name":"外部通道控制","mode":"slot","dmxFrom":0,"dmxTo":59,"dmxDefault":0},
        {"key":"auto.3","name":"自动 3","mode":"slot","dmxFrom":60,"dmxTo":84,"dmxDefault":60}]}]}}}),
    );
    let v = doc.view();
    edit(
        &mut doc,
        json!({"op":"addFixture","name":"程序灯","profileId":v.profiles.last().unwrap().id,
        "domainId":v.domains[0].id,"universe":1,"address":17}),
    );
    let id = doc.view().fixtures[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,
        "spaceId":null,"positionMeters":{"x":"1","y":"2","z":"4"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}}),
    );
    edit(&mut doc, json!({"op":"addScene","name":"自动程序"}));
    let scene_id = doc.view().scenes[0].id.clone();
    assert!(
        doc.edit(
            serde_json::from_value::<EditCommand>(
                json!({"op":"setSceneFunctionValue","sceneId":scene_id,"fixtureId":id,
        "attribute":"fixture-program","selection":{"functionKey":"auto.3","position":0}})
            )
            .unwrap()
        )
        .is_err()
    );
    edit(
        &mut doc,
        json!({"op":"setSceneFunctionValue","sceneId":scene_id,"fixtureId":id,
        "attribute":"fixture-program","selection":{"functionKey":"external","position":0}}),
    );
    let before = doc.encode().unwrap();
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.fixtures[0].light_simulation, "unmodeled-functions");
    assert_eq!(scene.fixtures[0].optics, "generic-illustrative");
    let compiled = doc.compile_scene(&scene_id).unwrap();
    let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
    player.execute(0, 0).unwrap();
    let out = compiled.output.render(player.values()).unwrap();
    assert_eq!(&out.slots[16..18], &[255, 0]);
    assert!(
        stagemaster_previs::editing_lights(&doc, Some(&scene_id)).unwrap()[0]
            .intensity
            .abs()
            < f64::EPSILON
    );
    assert!(
        stagemaster_previs::playback_lights(&doc, &out)[0]
            .intensity
            .abs()
            < f64::EPSILON
    );
    assert_eq!(doc.encode().unwrap(), before);
}
