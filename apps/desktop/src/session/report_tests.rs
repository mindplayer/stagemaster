use super::*;
#[test]
fn report_snapshot_preserves_history_generation_and_loaded_playback() {
    let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = serde_json::json!([]);
    let document = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let scene = document.view().scenes[0].id.clone();
    let mut s = Session::default();
    s.replace(document, None);
    s.preview(crate::preview::Request::LoadScene {
        generation: s.generation,
        scene_id: scene,
    })
    .unwrap();
    let before = serde_json::to_value(s.snapshot()).unwrap();
    let loaded = serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap())
        .unwrap()["loaded"]
        .clone();
    assert!(s.check_snapshot(s.generation - 1).is_err());
    let captured = s.check_snapshot(s.generation).unwrap();
    let report = captured.patch_report().unwrap();
    let sequence_id = captured.view().sequences[0].id.clone();
    let sequence = captured.sequence_report(&sequence_id).unwrap();
    assert_eq!(sequence.step_count(), 2);
    assert!(captured.sequence_report("已删除").is_err());
    assert_eq!(report.fixture_count(), 1);
    assert_eq!(before, serde_json::to_value(s.snapshot()).unwrap());
    assert_eq!(
        loaded,
        serde_json::to_value(s.preview(crate::preview::Request::Snapshot).unwrap()).unwrap()["loaded"]
    );
    assert!(s.export_source(s.generation).unwrap().is_none());
    s.edit(
        s.generation,
        EditCommand::SetInfo {
            name: "新编辑".into(),
            description: String::new(),
        },
    )
    .unwrap();
    assert_eq!(report.bytes(), captured.patch_report().unwrap().bytes());
    assert_eq!(
        sequence.bytes(),
        captured.sequence_report(&sequence_id).unwrap().bytes()
    );
    assert_ne!(
        sequence.bytes(),
        s.check_snapshot(s.generation)
            .unwrap()
            .sequence_report(&sequence_id)
            .unwrap()
            .bytes()
    );
    assert_ne!(
        report.bytes(),
        s.check_snapshot(s.generation)
            .unwrap()
            .patch_report()
            .unwrap()
            .bytes()
    );
    assert!(s.check_snapshot(s.generation - 1).is_err());
}
