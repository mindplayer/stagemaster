#[path = "support/fixture_function.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::Document;
use support::{choose, decode, definition, edit, raw, setup};
fn save(doc: &mut Document, definition: &Value) -> String {
    edit(
        doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition}}),
    )
    .unwrap();
    doc.view().profiles.last().unwrap().id.clone()
}
fn exchange(doc: &mut Document, ids: &[String], profile: &str, allow: bool) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"fixture","command":{"op":"exchange","fixtureIds":ids,
      "profileId":profile,"layout":null,"allowColorSlotRemap":allow}}),
    )
}
#[test]
fn confirmed_fixed_slot_revision_reencodes_only_selected_fixtures_and_keeps_preset_references() {
    let (mut doc, first, scenes) = setup(false);
    let view = doc.view();
    edit(
        &mut doc,
        json!({"op":"addFixture","name":"保持原版","profileId":view.fixtures[0].profile_id,
      "domainId":view.domains[0].id,"universe":1,"address":20}),
    )
    .unwrap();
    let second = doc.view().fixtures[1].id.clone();
    for fixture in [&first, &second] {
        choose(&mut doc, &scenes[0], fixture, "color-wheel", "red", 0).unwrap();
    }
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"recordPreset","name":"红色预设",
      "sceneId":scenes[0],"fixtureIds":[first,second],"attributes":["color-wheel"]}}),
    )
    .unwrap();
    let preset = doc.view().presets[0].id.clone();
    edit(
        &mut doc,
        json!({"op":"library","command":{"kind":"applyPreset","id":preset,
      "sceneId":scenes[1],"fixtureIds":[first,second],"attributes":["color-wheel"],"linked":true}}),
    )
    .unwrap();
    let mut def = definition(false);
    let red = &mut def["channels"][1]["functions"][1];
    red["dmxFrom"] = json!(48);
    red["dmxTo"] = json!(63);
    red["dmxDefault"] = json!(52);
    let target = save(&mut doc, &def);
    let before = doc.clone();
    assert!(exchange(&mut doc, std::slice::from_ref(&first), &target, false).is_err());
    assert_eq!(doc, before);
    exchange(&mut doc, std::slice::from_ref(&first), &target, true).unwrap();
    let old = raw(&before);
    let new = raw(&doc);
    for collection in ["profiles", "scenes", "presets", "patches"] {
        assert_eq!(old["lighting"][collection], new["lighting"][collection]);
    }
    for scene in scenes {
        let c = doc.compile_scene(&scene).unwrap();
        let frame = c.output.render(&c.plan.steps()[0].target).unwrap();
        assert_eq!(frame.slots[1], 52);
        assert_eq!(frame.slots[20], 20);
    }
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}
#[test]
fn opt_in_never_allows_range_mode_key_or_other_function_changes_and_batches_remain_atomic() {
    for (pointer, value) in [
        ("/channels/3/functions/2/dmxDefault", json!(65)),
        ("/channels/2/functions/1/dmxDefault", json!(26)),
        ("/channels/1/functions/1/key", json!("new-red")),
        ("/channels/1/functions/1/mode", json!("range")),
    ] {
        let (mut doc, id, _) = setup(false);
        let mut def = definition(false);
        *def.pointer_mut(pointer).unwrap() = value;
        let target = save(&mut doc, &def);
        let before = doc.clone();
        assert!(exchange(&mut doc, &[id], &target, true).is_err());
        assert_eq!(doc, before);
    }
    let (mut doc, id, _) = setup(false);
    let mut def = definition(false);
    def["channels"][1]["functions"][1]["dmxDefault"] = json!(21);
    let target = save(&mut doc, &def);
    let before = doc.clone();
    assert!(exchange(&mut doc, &[id, "missing-fixture".into()], &target, true).is_err());
    assert_eq!(doc, before);
}
#[test]
fn automatic_color_ranges_must_remain_identical_even_when_fixed_slots_migrate() {
    let (mut doc, id, _) = setup(false);
    let profile = doc.view().fixtures[0].profile_id.clone();
    let mut root = raw(&doc);
    let p = root["lighting"]["profiles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["id"] == profile)
        .unwrap();
    p["channels"][1]["functions"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"automatic","name":"自动换色",
      "mode":"range","dmxFrom":140,"dmxTo":255,"dmxDefault":140}));
    doc = decode(&root).unwrap();
    let mut def = serde_json::to_value(doc.profile_file(&profile).unwrap().definition()).unwrap();
    def["channels"][1]["functions"][1]["dmxDefault"] = json!(21);
    let target = save(&mut doc, &def);
    exchange(&mut doc, std::slice::from_ref(&id), &target, true).unwrap();
    def["channels"][1]["functions"][3]["dmxFrom"] = json!(141);
    def["channels"][1]["functions"][3]["dmxDefault"] = json!(141);
    let incompatible = save(&mut doc, &def);
    let before = doc.clone();
    assert!(exchange(&mut doc, &[id], &incompatible, true).is_err());
    assert_eq!(doc, before);
}
