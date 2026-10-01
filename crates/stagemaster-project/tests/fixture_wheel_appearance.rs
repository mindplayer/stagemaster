#[path = "support/fixture_function.rs"]
mod support;
use serde_json::{Value, json};
use stagemaster_project::{Document, ProfileFile};
use support::{choose, decode, definition, edit, raw, setup};

fn annotated() -> Value {
    let mut d = definition(false);
    d["name"] = json!("用户定制色盘");
    d["channels"][1]["functions"][0]["appearance"] = json!({"kind":"open"});
    d["channels"][1]["functions"][1]["appearance"] = json!({"kind":"color","colors":["#FF0033"]});
    d["channels"][1]["functions"][2]["appearance"] =
        json!({"kind":"color","colors":["#0000FF","#FFFFFF"]});
    d
}
fn save(doc: &mut Document, d: &Value) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":d}}),
    )
}
#[test]
fn annotations_round_trip_in_project_view_and_portable_mode_with_required_capability() {
    let mut doc = Document::new("色盘验收").unwrap();
    save(&mut doc, &annotated()).unwrap();
    let view = doc.view();
    let p = view.profiles.last().unwrap();
    let file = doc.profile_file(&p.id).unwrap();
    let imported = ProfileFile::decode(&file.encode().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&imported.definition().channels).unwrap(),
        serde_json::to_value(&p.channels).unwrap()
    );
    let mut other = Document::new("另一个工程").unwrap();
    save(
        &mut other,
        &serde_json::to_value(imported.definition()).unwrap(),
    )
    .unwrap();
    assert_ne!(other.view().profiles.last().unwrap().id, p.id);
    assert_eq!(decode(&raw(&doc)).unwrap(), doc);
    let mut root = raw(&doc);
    root["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "lighting.fixture-wheel-appearance");
    assert!(decode(&root).unwrap_err().contains("外观能力声明"));
}
#[test]
fn variant_changes_only_selected_fixture_and_preserves_scene_bytes_and_old_mode() {
    let (mut doc, id, scenes) = setup(false);
    let view = doc.view();
    let original = raw(&doc)["lighting"]["profiles"].clone();
    edit(&mut doc,json!({"op":"addFixture","name":"同批未改灯","profileId":view.fixtures[0].profile_id,"domainId":view.domains[0].id,"universe":1,"address":20})).unwrap();
    choose(&mut doc, &scenes[0], &id, "color-wheel", "red", 0).unwrap();
    let before = doc.compile_scene(&scenes[0]).unwrap();
    let old_scenes = raw(&doc)["lighting"]["scenes"].clone();
    let mut def = annotated();
    // A reversed/custom wheel changes its observed appearance at the same control slot.
    def["channels"][1]["functions"][1]["name"] = json!("实测为绿色");
    def["channels"][1]["functions"][1]["appearance"] = json!({"kind":"color","colors":["#00FF00"]});
    def["channels"][1]["functions"]
        .as_array_mut()
        .unwrap()
        .reverse();
    save(&mut doc, &def).unwrap();
    let variant = doc.view().profiles.last().unwrap().id.clone();
    edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[id],"profileId":variant,"layout":null}})).unwrap();
    let after = doc.compile_scene(&scenes[0]).unwrap();
    assert_eq!(before.plan, after.plan);
    assert_eq!(
        before
            .output
            .render(&before.plan.steps()[0].target)
            .unwrap()
            .slots,
        after
            .output
            .render(&after.plan.steps()[0].target)
            .unwrap()
            .slots
    );
    let root = raw(&doc);
    assert_eq!(root["lighting"]["scenes"], old_scenes);
    assert_eq!(
        root["lighting"]["profiles"].as_array().unwrap()[..original.as_array().unwrap().len()],
        *original.as_array().unwrap()
    );
    assert_eq!(
        doc.view().fixtures[1].profile_id,
        view.fixtures[0].profile_id
    );
    assert_eq!(doc.view().fixtures[0].profile_id, variant);
    assert_eq!(decode(&root).unwrap(), doc);
}
#[test]
fn annotations_never_allow_changed_control_mapping_or_mutate_used_modes() {
    let (mut doc, id, _) = setup(false);
    let old = doc.clone();
    let profile_id = doc.view().fixtures[0].profile_id.clone();
    assert!(edit(&mut doc,json!({"op":"fixture","command":{"op":"saveProfile","id":profile_id,"definition":annotated()}})).is_err());
    assert_eq!(doc, old);
    let mut def = annotated();
    def["channels"][1]["functions"][1]["dmxDefault"] = json!(21);
    save(&mut doc, &def).unwrap();
    let variant = doc.view().profiles.last().unwrap().id.clone();
    let before = doc.clone();
    assert!(edit(&mut doc,json!({"op":"fixture","command":{"op":"exchange","fixtureIds":[id],"profileId":variant,"layout":null}})).is_err());
    assert_eq!(doc, before);
}
#[test]
fn invalid_appearance_or_wrong_attribute_is_rejected_without_partial_edits() {
    let mut doc = Document::new("严格建档").unwrap();
    let before = doc.clone();
    for value in [
        json!({"kind":"color","colors":[]}),
        json!({"kind":"color","colors":["red"]}),
        json!({"kind":"color","colors":["#ffffff","#000000","#abcdef"]}),
    ] {
        let mut def = annotated();
        def["channels"][1]["functions"][1]["appearance"] = value;
        assert!(save(&mut doc, &def).is_err());
        assert_eq!(doc, before);
    }
    for (channel, function) in [(2, 0), (3, 2), (4, 0)] {
        let mut def = annotated();
        def["channels"][channel]["functions"][function]["appearance"] = json!({"kind":"open"});
        assert!(save(&mut doc, &def).is_err());
        assert_eq!(doc, before);
    }
    let mut def = annotated();
    def["channels"][1]["functions"][1]["mode"] = json!("range");
    assert!(save(&mut doc, &def).is_err());
    assert_eq!(doc, before);
    let mut malformed = annotated();
    malformed["channels"][1]["functions"][1]["appearance"] =
        json!({"kind":"open","colors":["#ffffff"]});
    assert!(serde_json::from_value::<stagemaster_project::ProfileDefinition>(malformed).is_err());
}
