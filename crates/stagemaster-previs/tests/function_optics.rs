use serde_json::json;
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, value: serde_json::Value) {
    doc.edit(serde_json::from_value::<EditCommand>(value).unwrap())
        .unwrap();
}
#[test]
fn unsupported_function_optics_keep_geometry_without_claiming_light_output() {
    let mut doc = Document::new("功能光学校验").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":{
        "name":"快门测试","manufacturer":"契约","model":"测试","mode":"双通道","footprint":2,
        "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":65535},
        {"attribute":"shutter","coarse":2,"fine":null,"defaultValue":{"functionKey":"open","position":0},
        "functions":[{"key":"open","name":"常开","mode":"slot","dmxFrom":0,"dmxTo":255,"dmxDefault":20}]}]}}}),
    );
    let v = doc.view();
    edit(
        &mut doc,
        json!({"op":"addFixture","name":"功能灯","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":1}),
    );
    let id = doc.view().fixtures[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":"1","y":"2","z":"4"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}}),
    );
    edit(&mut doc, json!({"op":"addScene","name":"测试"}));
    let before = doc.encode().unwrap();
    let scene = stagemaster_previs::scene(&doc).unwrap();
    assert_eq!(scene.fixtures.len(), 1);
    assert_eq!(scene.fixtures[0].optics, "generic-illustrative");
    assert_eq!(scene.fixtures[0].light_simulation, "unmodeled-functions");
    assert!(
        scene.fixtures[0]
            .origin_meters
            .iter()
            .zip([1.0, 2.0, 4.0])
            .all(|(a, b)| (*a - b).abs() < f64::EPSILON)
    );
    let compiled = doc.compile_scene(&doc.view().scenes[0].id).unwrap();
    let out = compiled.output.render(compiled.plan.defaults()).unwrap();
    assert_eq!(&out.slots[..2], &[255, 20]);
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
    assert_eq!(before, doc.encode().unwrap());
}
