use serde_json::{Value, json};
use stagemaster_project::Document;
fn document(count: usize) -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let base = root["lighting"]["sequences"][0]["steps"][0].clone();
    root["lighting"]["sequences"][0]["steps"] = (0..count)
        .map(|i| {
            let mut step = base.clone();
            step["id"] = format!("70000000-0000-4000-8000-{i:012}").into();
            step["number"] = (i + 1).to_string().into();
            step["name"] = format!("步骤{i}").into();
            step
        })
        .collect();
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn apply(doc: &mut Document, ids: &[String], op: Value) -> Result<(), String> {
    let id = doc.view().sequences[0].id.clone();
    let mut command = json!({"op":"sequence","command":{"kind":"editSteps","id":id,"stepIds":ids}});
    command["command"]["operation"] = op;
    doc.edit(serde_json::from_value(command).unwrap())
}
fn ids(doc: &Document) -> Vec<String> {
    doc.view().sequences[0]
        .steps
        .iter()
        .map(|s| s.id.clone())
        .collect()
}
#[test]
fn copy_normalizes_order_preserves_content_and_gets_independent_identity() {
    let mut doc = document(4);
    let v = doc.view();
    let seq = &v.sequences[0];
    doc.edit(serde_json::from_value(json!({"op":"sequence","command":{"kind":"updateStepScript","id":seq.id,"stepId":seq.steps[0].id,"script":{"section":"第一幕","trigger":"演员举手","notes":"等掌声结束"}}})).unwrap()).unwrap();
    let loaded = doc.compile_sequence(&seq.id).unwrap();
    let original: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let selected = vec![seq.steps[2].id.clone(), seq.steps[0].id.clone()];
    apply(
        &mut doc,
        &selected,
        json!({"kind":"copy","beforeId":seq.steps[1].id}),
    )
    .unwrap();
    let after: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let steps = after["lighting"]["sequences"][0]["steps"]
        .as_array()
        .unwrap();
    for (source, destination, number) in [(0, 1, "5"), (2, 2, "6")] {
        let mut expected = original["lighting"]["sequences"][0]["steps"][source].clone();
        assert_ne!(expected["id"], steps[destination]["id"]);
        expected["id"] = steps[destination]["id"].clone();
        expected["number"] = number.into();
        assert_eq!(expected, steps[destination]);
    }
    assert_eq!(loaded.steps.len(), 4);
    assert_eq!(loaded.steps[0].script.as_ref().unwrap().trigger, "演员举手");
    assert_eq!(doc.compile_sequence(&seq.id).unwrap().steps.len(), 6);
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn move_and_remove_are_atomic_and_noop_keeps_exact_document() {
    let mut doc = document(4);
    let original = doc.clone();
    let ids = ids(&doc);
    apply(
        &mut doc,
        &ids[1..3],
        json!({"kind":"move","beforeId":ids[3]}),
    )
    .unwrap();
    assert_eq!(doc, original);
    apply(
        &mut doc,
        &[ids[3].clone(), ids[1].clone()],
        json!({"kind":"move","beforeId":ids[0]}),
    )
    .unwrap();
    assert_eq!(
        self::ids(&doc),
        vec![
            ids[1].clone(),
            ids[3].clone(),
            ids[0].clone(),
            ids[2].clone()
        ]
    );
    apply(
        &mut doc,
        &[ids[3].clone(), ids[1].clone()],
        json!({"kind":"move","beforeId":null}),
    )
    .unwrap();
    assert_eq!(
        self::ids(&doc),
        vec![
            ids[0].clone(),
            ids[2].clone(),
            ids[1].clone(),
            ids[3].clone()
        ]
    );
    apply(
        &mut doc,
        &[ids[0].clone(), ids[2].clone()],
        json!({"kind":"remove"}),
    )
    .unwrap();
    assert_eq!(self::ids(&doc), vec![ids[1].clone(), ids[3].clone()]);
}
#[test]
fn rejects_unknown_duplicate_empty_all_deleted_and_selected_destination() {
    let mut doc = document(4);
    let before = doc.clone();
    let ids = ids(&doc);
    for (selected, op) in [
        (vec![], json!({"kind":"remove"})),
        (
            vec![ids[0].clone(), ids[0].clone()],
            json!({"kind":"copy","beforeId":null}),
        ),
        (vec!["missing".into()], json!({"kind":"remove"})),
        (ids.clone(), json!({"kind":"remove"})),
        (
            vec![ids[0].clone()],
            json!({"kind":"move","beforeId":ids[0]}),
        ),
        (
            vec![ids[0].clone()],
            json!({"kind":"move","beforeId":"missing"}),
        ),
        (
            vec![ids[0].clone()],
            json!({"kind":"copy","beforeId":"missing"}),
        ),
    ] {
        assert!(apply(&mut doc, &selected, op).is_err());
        assert_eq!(doc, before);
    }
}
#[test]
fn copy_capacity_and_script_budget_roll_back_entire_group() {
    let mut doc = document(1023);
    let selected = ids(&doc)[..2].to_vec();
    let before = doc.clone();
    assert!(
        apply(&mut doc, &selected, json!({"kind":"copy","beforeId":null}))
            .unwrap_err()
            .contains("1024")
    );
    assert_eq!(doc, before);
    apply(
        &mut doc,
        &selected[..1],
        json!({"kind":"copy","beforeId":null}),
    )
    .unwrap();
    assert_eq!(ids(&doc).len(), 1024);
    let mut doc = document(4);
    let v = doc.view();
    let seq = &v.sequences[0];
    for step in &seq.steps {
        doc.edit(serde_json::from_value(json!({"op":"sequence","command":{"kind":"updateStepScript","id":seq.id,"stepId":step.id,"script":{"section":"","trigger":"","notes":"字".repeat(4096)}}})).unwrap()).unwrap();
    }
    let before = doc.clone();
    let selected = ids(&doc)[..2].to_vec();
    assert!(
        apply(&mut doc, &selected, json!({"kind":"copy","beforeId":null}))
            .unwrap_err()
            .contains("64 KiB")
    );
    assert_eq!(doc, before);
}
