use serde_json::{Value, json};
use stagemaster_playback::{MAX_STEPS, MAX_TARGET_VALUES};
use stagemaster_project::{CheckLocation, Document, ProgramStatus, Severity};

fn root() -> Value {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    root
}
fn decode(root: &Value) -> Document {
    Document::decode(&serde_json::to_vec(root).unwrap()).unwrap()
}

#[test]
fn empty_project_never_passes_without_a_program_and_output() {
    let report = Document::new("空工程").unwrap().check();
    assert!(!report.desktop_ready);
    assert!(report.programs.is_empty());
    assert_eq!(
        report.issues.iter().map(|i| i.code).collect::<Vec<_>>(),
        ["patch.empty", "program.empty"]
    );
}

#[test]
fn aggregates_every_missing_patch_and_blocks_programs_without_duplicate_errors() {
    let mut root = root();
    root["lighting"]["patches"] = json!([]);
    let doc = decode(&root);
    let report = doc.check();
    let failures: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert_eq!(failures.len(), doc.view().fixtures.len());
    for fixture in doc.view().fixtures {
        assert!(failures.iter().any(|i| i.code == "patch.missing"
            && i.location
                == CheckLocation::Fixture {
                    id: fixture.id.clone()
                }));
    }
    assert!(!report.desktop_ready);
    assert!(
        report
            .programs
            .iter()
            .all(|p| p.status == ProgramStatus::Blocked && p.usage.is_none())
    );
    assert!(doc.compile_scene(&doc.view().scenes[0].id).is_err());
}

#[test]
fn multiple_output_lines_are_locatable_and_match_compiler_restriction() {
    let mut root = root();
    // Add a second fixture with no assignments. Even unused fixtures participate in output.
    let mut fixture = root["lighting"]["fixtures"][0].clone();
    fixture["id"] = json!("29999999-0000-4000-8000-000000000111");
    fixture["name"] = json!("第二条线路");
    let mut patch = root["lighting"]["patches"][0].clone();
    patch["fixtureId"] = fixture["id"].clone();
    patch["universe"] = json!(2);
    root["lighting"]["fixtures"]
        .as_array_mut()
        .unwrap()
        .push(fixture);
    root["lighting"]["patches"]
        .as_array_mut()
        .unwrap()
        .push(patch);
    let doc = decode(&root);
    let report = doc.check();
    assert!(!report.desktop_ready);
    assert_eq!(
        report
            .issues
            .iter()
            .filter(|i| i.code == "patch.multipleLines")
            .count(),
        doc.view().fixtures.len()
    );
    assert!(doc.compile_scene(&doc.view().scenes[0].id).is_err());
}

#[test]
fn valid_report_is_read_only_roundtrips_and_does_not_require_3d_placement() {
    let doc = decode(&root());
    let before = doc.encode().unwrap();
    let report = doc.check();
    assert!(report.desktop_ready);
    assert_eq!(
        report.programs.len(),
        doc.view().scenes.len() + doc.view().sequences.len()
    );
    assert!(
        report
            .issues
            .iter()
            .all(|i| i.severity == Severity::Warning && i.code == "stage.unplaced")
    );
    let seq = report.programs.last().unwrap().usage.as_ref().unwrap();
    assert_eq!(seq.attributes, 4);
    assert_eq!(seq.steps, 2);
    assert_eq!(seq.target_values, 8);
    assert_eq!(seq.value_buffer_bytes, 40);
    assert_eq!(seq.effect_buffer_bytes, 0);
    assert_eq!(report.limits.steps, MAX_STEPS);
    assert_eq!(report.limits.target_values, MAX_TARGET_VALUES);
    assert_eq!(before, doc.encode().unwrap());
    let saved = doc.next_revision();
    let reopened = Document::decode(&saved.encode().unwrap()).unwrap().check();
    assert!(reopened.desktop_ready);
    assert_eq!(report.project_id, reopened.project_id);
    assert_ne!(report.revision_id, reopened.revision_id);
    assert_eq!(
        serde_json::to_value(&report.programs).unwrap(),
        serde_json::to_value(&reopened.programs).unwrap()
    );
}

#[test]
fn one_oversized_list_does_not_hide_other_results_and_repair_rechecks() {
    let mut root = root();
    let mut bad = root["lighting"]["sequences"][0].clone();
    bad["id"] = json!("29999999-0000-4000-8000-000000000222");
    bad["name"] = json!("超量列表");
    let template = bad["steps"][0].clone();
    bad["steps"] = (0..=MAX_STEPS)
        .map(|n| {
            let mut step = template.clone();
            step["id"] = json!(format!("39999999-0000-4000-8000-{n:012x}"));
            step["number"] = json!((n + 1).to_string());
            step
        })
        .collect();
    // Put bad first, proving this is not a fail-fast pass over programs.
    root["lighting"]["sequences"]
        .as_array_mut()
        .unwrap()
        .insert(0, bad);
    let report = decode(&root).check();
    assert!(!report.desktop_ready);
    assert_eq!(report.issues[0].severity, Severity::Error);
    assert_eq!(
        report
            .programs
            .iter()
            .filter(|p| p.status == ProgramStatus::Failed)
            .count(),
        1
    );
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.message.contains("1025 / 1024"))
    );
    assert_eq!(
        report.programs.last().unwrap().status,
        ProgramStatus::Passed
    );
    assert!(report.issues.iter().any(|i| i.code == "program.compile"
        && i.location
            == CheckLocation::Sequence {
                id: "29999999-0000-4000-8000-000000000222".into()
            }));
    root["lighting"]["sequences"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .truncate(MAX_STEPS);
    assert!(decode(&root).check().desktop_ready);
}

#[test]
fn effect_and_keyframe_stats_count_compiled_instances_and_ignore_disabled_effects() {
    let mut doc = decode(&root());
    let view = doc.view();
    let scene = &view.scenes[0].id;
    doc.edit(serde_json::from_value(json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
        "id":"29999999-0000-4000-8000-000000000333","name":"测试关键帧","enabled":true,
        "fixtureIds":[view.fixtures[0].id],"periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,
        "reverse":false,"waveform":"keyframes","dutyPercent":50,
        "channels":[{"attribute":"dimmer","keyframes":[{"position":0,"value":0,"transition":"linear"},{"position":5000,"value":65535,"transition":"smooth"}]}]
    }}})).unwrap()).unwrap();
    let report = doc.check();
    assert!(report.desktop_ready);
    let usage = report.programs[0].usage.as_ref().unwrap();
    assert_eq!((usage.effect_channels, usage.keyframes), (1, 2));
    assert!(usage.effect_buffer_bytes > 0);
    let mut saved: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    saved["lighting"]["scenes"][0]["effects"][0]["enabled"] = json!(false);
    let report = decode(&saved).check();
    let usage = report.programs[0].usage.as_ref().unwrap();
    assert_eq!(
        (
            usage.effect_channels,
            usage.keyframes,
            usage.effect_buffer_bytes
        ),
        (0, 0, 0)
    );
}
