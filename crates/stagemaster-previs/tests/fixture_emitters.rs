use serde_json::json;
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, value: serde_json::Value) {
    doc.edit(serde_json::from_value::<EditCommand>(value).unwrap())
        .unwrap();
}
#[test]
fn independent_emitters_keep_body_and_hide_unmodeled_editing_and_playback_beams() {
    let mut doc = Document::new("独立光源预演边界").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":{
      "name":"双光源","manufacturer":"测试","model":"软件子集","mode":"非实灯档案","footprint":2,
      "emitters":[{"key":"pattern","name":"图案"},{"key":"wash","name":"染色"}],
      "channels":[{"attribute":"emitter.pattern.dimmer","coarse":1,"fine":null,"defaultValue":65535},
        {"attribute":"emitter.wash.dimmer","coarse":2,"fine":null,"defaultValue":65535}]}}}),
    );
    let view = doc.view();
    edit(
        &mut doc,
        json!({"op":"addFixture","name":"两光源灯","profileId":view.profiles.last().unwrap().id,"domainId":view.domains[0].id,"universe":1,"address":17}),
    );
    let id = doc.view().fixtures[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":"1","y":"2","z":"4"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}}),
    );
    edit(&mut doc, json!({"op":"addScene","name":"全亮"}));
    let before = doc.clone();
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].light_simulation, "unmodeled-emitters");
    let compiled = doc.compile_scene(&doc.view().scenes[0].id).unwrap();
    let out = compiled.output.render(compiled.plan.defaults()).unwrap();
    assert_eq!(&out.slots[16..18], &[255, 255]);
    assert!(
        stagemaster_previs::editing_lights(&doc, None).unwrap()[0]
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
    assert_eq!(doc, before);
}
