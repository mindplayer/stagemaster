use serde_json::json;
use stagemaster_project::{Document, EditCommand};
fn edit(d: &mut Document, value: serde_json::Value) {
    d.edit(serde_json::from_value::<EditCommand>(value).unwrap())
        .unwrap();
}
#[test]
fn continuous_optics_keep_fixture_geometry_without_claiming_generic_beam_accuracy() {
    for key in ["zoom", "focus", "iris"] {
        let mut d = Document::new("镜头预演边界").unwrap();
        edit(
            &mut d,
            json!({"op":"fixture","command":{"op":"saveProfile","definition":{
            "name":"镜头模式","manufacturer":"测试","model":"镜头","mode":"双通道","footprint":2,
            "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":65535},
              {"attribute":key,"coarse":2,"fine":null,"defaultValue":65535}]}}}),
        );
        let v = d.view();
        edit(
            &mut d,
            json!({"op":"addFixture","name":"镜头灯","profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":1}),
        );
        let id = d.view().fixtures[0].id.clone();
        edit(
            &mut d,
            json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":"1","y":"2","z":"4"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}}),
        );
        edit(&mut d, json!({"op":"addScene","name":"全控制值"}));
        let before = d.clone();
        let scene = stagemaster_previs::scene(&d).unwrap();
        assert_eq!(scene.fixtures.len(), 1);
        assert_eq!(scene.fixtures[0].light_simulation, "unmodeled-optics");
        let c = d.compile_scene(&d.view().scenes[0].id).unwrap();
        let out = c.output.render(c.plan.defaults()).unwrap();
        assert_eq!(&out.slots[..2], &[255, 255]);
        assert!(
            stagemaster_previs::editing_lights(&d, None).unwrap()[0]
                .intensity
                .abs()
                < f64::EPSILON
        );
        assert!(
            stagemaster_previs::playback_lights(&d, &out)[0]
                .intensity
                .abs()
                < f64::EPSILON
        );
        assert_eq!(d, before);
    }
}
