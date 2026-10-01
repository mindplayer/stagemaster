use serde_json::{Value, json};
use stagemaster_project::Document;
fn document() -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn edit(doc: &mut Document, ids: &[String], patch: Value) -> Result<(), String> {
    let mut cmd = json!({"op":"sequence","command":{"kind":"editSteps","id":doc.view().sequences[0].id,"stepIds":ids,"operation":{"kind":"timing"}}});
    cmd["command"]["operation"]["patch"] = patch;
    let parsed = serde_json::from_value(cmd).map_err(|e| e.to_string())?;
    doc.edit(parsed)
}
#[test]
fn sparse_changes_preserve_unselected_steps_metadata_and_other_timing() {
    let mut doc = document();
    let v = doc.view();
    let seq = &v.sequences[0];
    let first = &seq.steps[0];
    doc.edit(serde_json::from_value(json!({"op":"sequence","command":{"kind":"updateStepScript","id":seq.id,"stepId":first.id,"script":{"section":"第一幕","trigger":"抬手","notes":"待掌声结束"}}})).unwrap()).unwrap();
    let original: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    edit(
        &mut doc,
        std::slice::from_ref(&first.id),
        json!({"fadeMs":1234}),
    )
    .unwrap();
    let mut expected = original;
    expected["lighting"]["sequences"][0]["steps"][0]["fade"] =
        json!({"ticks":"1234","ticksPerSecond":"1000"});
    let actual: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn group_manual_auto_zero_and_upper_bound_keep_loaded_plan_independent() {
    let mut doc = document();
    let v = doc.view();
    let seq = &v.sequences[0];
    let ids: Vec<_> = seq.steps.iter().map(|s| s.id.clone()).collect();
    let loaded = doc.compile_sequence(&seq.id).unwrap();
    edit(
        &mut doc,
        &ids,
        json!({"delayMs":0,"fadeMs":86_400_000,"advance":{"kind":"after","waitMs":17}}),
    )
    .unwrap();
    for step in &doc.view().sequences[0].steps {
        assert_eq!(step.delay_ms, 0);
        assert_eq!(step.fade_ms, 86_400_000);
        assert_eq!(step.wait_ms, Some(17));
    }
    assert_ne!(loaded.plan, doc.compile_sequence(&seq.id).unwrap().plan);
    edit(&mut doc, &ids, json!({"advance":{"kind":"manual"}})).unwrap();
    for step in &doc.view().sequences[0].steps {
        assert!(step.wait_ms.is_none());
        assert_eq!(step.fade_ms, 86_400_000);
    }
    let before = doc.clone();
    edit(&mut doc, &ids, json!({"delayMs":0})).unwrap();
    assert_eq!(doc, before);
}
#[test]
fn invalid_patch_rejects_before_any_step_changes() {
    let mut doc = document();
    let ids: Vec<_> = doc.view().sequences[0]
        .steps
        .iter()
        .map(|s| s.id.clone())
        .collect();
    let before = doc.clone();
    for patch in [
        json!({}),
        json!({"delayMs":86_400_001}),
        json!({"fadeMs":-1}),
        json!({"fadeMs":1.5}),
        json!({"fadeMs":1,"advance":{"kind":"after","waitMs":86_400_001}}),
        json!({"fadeMs":1,"advance":{"kind":"manual","waitMs":0}}),
        json!({"unknown":0}),
        json!({"advance":{"kind":"after"}}),
    ] {
        let label = patch.to_string();
        assert!(edit(&mut doc, &ids, patch).is_err(), "accepted {label}");
        assert_eq!(doc, before);
    }
    for selected in [
        vec![],
        vec![ids[0].clone(), ids[0].clone()],
        vec![ids[0].clone(), "missing".into()],
    ] {
        assert!(edit(&mut doc, &selected, json!({"delayMs":1})).is_err());
        assert_eq!(doc, before);
    }
}

#[test]
fn structural_remove_also_rejects_unused_fields_instead_of_ignoring_them() {
    assert!(
        serde_json::from_value::<stagemaster_project::StepGroupOperation>(
            json!({"kind":"remove","beforeId":null})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<stagemaster_project::StepGroupOperation>(json!({"kind":"remove"}))
            .is_ok()
    );
}
