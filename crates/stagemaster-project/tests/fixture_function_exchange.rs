#[path = "support/fixture_function.rs"]
mod support;
use serde_json::json;
use support::{choose, decode, definition, edit, raw, setup};

#[test]
fn copying_and_safe_exchange_preserve_function_identity_and_only_reencode_physical_channels() {
    let (mut doc, id, scenes) = setup(false);
    let view = doc.view();
    edit(&mut doc, json!({"op":"addFixture","name":"接收复制","profileId":view.fixtures[0].profile_id,"domainId":view.domains[0].id,"universe":1,"address":20})).unwrap();
    let destination = doc.view().fixtures[1].id.clone();
    choose(&mut doc, &scenes[0], &id, "shutter", "strobe", 41234).unwrap();
    choose(&mut doc, &scenes[0], &id, "color-wheel", "red", 0).unwrap();
    edit(&mut doc, json!({"op":"library","command":{"kind":"copyValues","sceneId":scenes[0],"sourceId":id,"fixtureIds":[destination],"attributes":["shutter","color-wheel"]}})).unwrap();
    edit(
        &mut doc,
        json!({"op":"duplicateScene","id":scenes[0],"name":"复制"}),
    )
    .unwrap();
    let before = raw(&doc)["lighting"]["scenes"].clone();
    let mut revised = definition(true);
    revised["name"] = json!("粗细交换模式");
    revised["channels"][1]["coarse"] = json!(6);
    revised["channels"][1]["fine"] = json!(2);
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":revised}}),
    )
    .unwrap();
    let profile = doc.view().profiles.last().unwrap().id.clone();
    edit(&mut doc, json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[id],"profileId":profile,"layout":null}})).unwrap();
    assert_eq!(raw(&doc)["lighting"]["scenes"], before);
    assert_eq!(decode(&raw(&doc)).unwrap(), doc);
    for scene in doc.view().scenes.iter().filter(|s| s.id != scenes[1]) {
        for attribute in ["shutter", "color-wheel"] {
            let source = scene
                .values
                .iter()
                .find(|v| v.fixture_id == id && v.attribute == attribute)
                .unwrap();
            let copied = scene
                .values
                .iter()
                .find(|v| v.fixture_id == destination && v.attribute == attribute)
                .unwrap();
            assert_eq!(source.function_value, copied.function_value);
        }
        let compiled = doc.compile_scene(&scene.id).unwrap();
        let out = compiled
            .output
            .render(&compiled.plan.steps()[0].target)
            .unwrap();
        assert_eq!(out.slots[1], 20);
        assert_eq!(out.slots[5], 0);
        let wheel = out.fixtures[0]
            .attributes
            .iter()
            .find(|a| a.key == "color-wheel")
            .unwrap()
            .function
            .as_ref()
            .unwrap();
        assert_eq!(wheel.key, "red");
        assert_eq!(wheel.name, "红色");
        assert_eq!(wheel.dmx_value, 20);
        assert_eq!(wheel.position, None);
    }
    revised["channels"][1]["functions"][1]["dmxDefault"] = json!(21);
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":revised}}),
    )
    .unwrap();
    let newer = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    assert!(edit(&mut doc, json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[id],"profileId":newer,"layout":null}})).unwrap_err().contains("功能区间"));
    assert_eq!(doc, before);
}
