use serde_json::{Value, json};
use stagemaster_project::Document;
fn document(count: usize) -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let sample = root["lighting"]["sequences"][0]["steps"][0].clone();
    root["lighting"]["sequences"][0]["steps"] = (0..count)
        .map(|i| {
            let mut step = sample.clone();
            step["id"] = format!("aaaaaaaa-0000-4000-8000-{i:012}").into();
            step["number"] = (i + 1).to_string().into();
            step
        })
        .collect::<Vec<_>>()
        .into();
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn ids(doc: &Document) -> Vec<String> {
    doc.view().sequences[0]
        .steps
        .iter()
        .map(|s| s.id.clone())
        .collect()
}
fn edit(doc: &mut Document, selected: &[String], patch: &Value) -> Result<(), String> {
    let command = json!({"op":"sequence","command":{"kind":"editSteps","id":doc.view().sequences[0].id,"stepIds":selected,"operation":{"kind":"script","patch":patch}}});
    doc.edit(serde_json::from_value(command).map_err(|e| e.to_string())?)
}
fn raw(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
#[test]
fn sparse_notes_preserve_each_trigger_timing_identity_and_unselected_content() {
    let mut doc = document(3);
    let selected = ids(&doc);
    for (i, id) in selected.iter().enumerate() {
        edit(
            &mut doc,
            std::slice::from_ref(id),
            &json!({"section":"旧幕","trigger":format!("台词 {i}\n动作"),"notes":"个别备注"}),
        )
        .unwrap();
    }
    let before = raw(&doc);
    edit(
        &mut doc,
        &[selected[2].clone(), selected[0].clone()],
        &json!({"section":"第二幕","notes":"等掌声\n再执行"}),
    )
    .unwrap();
    let mut expected = before;
    for index in [0, 2] {
        expected["lighting"]["sequences"][0]["steps"][index]["script"]["section"] = "第二幕".into();
        expected["lighting"]["sequences"][0]["steps"][index]["script"]["notes"] =
            "等掌声\n再执行".into();
    }
    assert_eq!(raw(&doc), expected);
    assert_eq!(Document::decode(&doc.encode().unwrap()).unwrap(), doc);
}
#[test]
fn clear_removes_only_chosen_fields_and_capability_after_the_last_script() {
    let mut doc = document(2);
    let selected = ids(&doc);
    edit(
        &mut doc,
        &selected,
        &json!({"section":"第一幕","trigger":"举手"}),
    )
    .unwrap();
    edit(&mut doc, &selected[..1], &json!({"section":""})).unwrap();
    assert_eq!(
        doc.view().sequences[0].steps[0]
            .script
            .as_ref()
            .unwrap()
            .trigger,
        "举手"
    );
    edit(&mut doc, &selected[..1], &json!({"trigger":""})).unwrap();
    assert!(doc.view().sequences[0].steps[0].script.is_none());
    assert!(
        raw(&doc)["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "lighting.sequence-script")
    );
    edit(
        &mut doc,
        &selected[1..],
        &json!({"section":"","trigger":""}),
    )
    .unwrap();
    assert!(
        !raw(&doc)["requires"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "lighting.sequence-script")
    );
    let clean = doc.clone();
    edit(&mut doc, &selected, &json!({"section":""})).unwrap();
    assert_eq!(doc, clean);
}
#[test]
fn malformed_patch_or_selection_never_partially_changes_steps() {
    let mut doc = document(3);
    let selected = ids(&doc);
    let before = doc.clone();
    for patch in [
        &json!({}),
        &json!({"section":null,"notes":"valid"}),
        &json!({"notes":true}),
        &json!({"unknown":"x"}),
        &json!({"section":"🎭".repeat(81)}),
        &json!({"trigger":"字".repeat(1025)}),
        &json!({"notes":"字".repeat(4097)}),
        &json!({"section":"valid","notes":"\u{0000}"}),
        &json!({"trigger":"\u{0085}"}),
    ] {
        assert!(edit(&mut doc, &selected, patch).is_err());
        assert_eq!(doc, before);
    }
    for chosen in [
        vec![],
        vec![selected[0].clone(), selected[0].clone()],
        vec![selected[0].clone(), "missing".into()],
    ] {
        assert!(edit(&mut doc, &chosen, &json!({"section":"第一幕"})).is_err());
        assert_eq!(doc, before);
    }
}
#[test]
fn unicode_and_total_utf8_budget_include_unchanged_steps_and_clear_before_budgeting() {
    let mut doc = document(17);
    let selected = ids(&doc);
    edit(
        &mut doc,
        &selected[..16],
        &json!({"notes":"🎭".repeat(1024)}),
    )
    .unwrap();
    let before = doc.clone();
    assert!(
        edit(&mut doc, &selected[16..], &json!({"notes":"x"}))
            .unwrap_err()
            .contains("64 KiB")
    );
    assert_eq!(doc, before);
    edit(
        &mut doc,
        &selected[..16],
        &json!({"notes":" ".repeat(4096)}),
    )
    .unwrap();
    assert!(
        doc.view().sequences[0]
            .steps
            .iter()
            .all(|s| s.script.is_none())
    );
    edit(
        &mut doc,
        &selected[..1],
        &json!({"section":"🎭".repeat(80)}),
    )
    .unwrap();
}
#[test]
fn script_changes_leave_playback_plan_and_loaded_metadata_independent() {
    let mut doc = document(3);
    let selected = ids(&doc);
    let sequence = doc.view().sequences[0].id.clone();
    let loaded = doc.compile_sequence(&sequence).unwrap();
    edit(
        &mut doc,
        &selected,
        &json!({"section":"新幕","trigger":"不要自动执行"}),
    )
    .unwrap();
    let revised = doc.compile_sequence(&sequence).unwrap();
    assert_eq!(loaded.plan, revised.plan);
    assert!(loaded.steps.iter().all(|s| s.script.is_none()));
    assert!(
        revised
            .steps
            .iter()
            .all(|s| s.script.as_ref().unwrap().section == "新幕")
    );
}
