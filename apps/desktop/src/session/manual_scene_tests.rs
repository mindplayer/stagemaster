use super::*;
use std::fmt::Write;
#[test]
fn manual_recording_is_one_history_change_and_stale_generation_is_rejected() {
    let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    json["entryPoints"] = serde_json::json!([]);
    let document = Document::decode(&serde_json::to_vec(&json).unwrap()).unwrap();
    let view = document.view();
    let compiled = document.compile_live_scene(&view.scenes[0].id, 0).unwrap();
    let layout = compiled
        .layout()
        .id()
        .iter()
        .fold(String::new(), |mut text, b| {
            write!(text, "{b:02x}").unwrap();
            text
        });
    let capture = document
        .capture_manual_scene(
            &layout,
            vec![stagemaster_project::ManualSceneReading {
                fixture_id: view.fixtures[0].id.clone(),
                attribute: "dimmer".into(),
                value: 0,
            }],
        )
        .unwrap();
    let mut session = Session::default();
    session.replace(document.clone(), None);
    assert!(
        session
            .record_manual_scene(session.generation - 1, &capture, "场景")
            .is_err()
    );
    assert!(session.undo.is_empty());
    session
        .record_manual_scene(session.generation, &capture, "记录")
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after = session.document.clone().unwrap();
    session.history(session.generation, false).unwrap();
    assert!(session.document.as_ref().unwrap().same_content(&document));
    session.history(session.generation, true).unwrap();
    assert!(session.document.as_ref().unwrap().same_content(&after));
}

#[test]
fn manual_merge_has_atomic_history_and_identical_merge_does_not_add_history() {
    let mut document = Document::new("合并历史").unwrap();
    let view = document.view();
    document
        .edit(EditCommand::AddFixture {
            name: "灯".into(),
            profile_id: view.profiles[0].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address: 1,
        })
        .unwrap();
    document
        .edit(EditCommand::AddScene {
            name: "原场景".into(),
        })
        .unwrap();
    let view = document.view();
    document
        .edit(EditCommand::SetSceneValue {
            scene_id: view.scenes[0].id.clone(),
            fixture_id: view.fixtures[0].id.clone(),
            attribute: "dimmer".into(),
            mode: stagemaster_project::ValueMode::Literal,
            value: 65535,
        })
        .unwrap();
    let compiled = document.compile_live_scene(&view.scenes[0].id, 0).unwrap();
    let layout = compiled
        .layout()
        .id()
        .iter()
        .fold(String::new(), |mut s, b| {
            write!(s, "{b:02x}").unwrap();
            s
        });
    let capture = document
        .capture_manual_scene(
            &layout,
            vec![stagemaster_project::ManualSceneReading {
                fixture_id: view.fixtures[0].id.clone(),
                attribute: "dimmer".into(),
                value: 0,
            }],
        )
        .unwrap();
    let merge = document
        .prepare_manual_scene_merge(&capture, &view.scenes[0].id)
        .unwrap();
    let mut session = Session::default();
    session.replace(document.clone(), None);
    assert!(
        session
            .merge_manual_scene(session.generation - 1, &merge)
            .is_err()
    );
    assert!(session.undo.is_empty());
    session
        .merge_manual_scene(session.generation, &merge)
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after = session.document.clone().unwrap();
    let same = after
        .prepare_manual_scene_merge(&capture, &view.scenes[0].id)
        .unwrap();
    let generation = session.generation;
    session.merge_manual_scene(generation, &same).unwrap();
    assert_eq!(session.generation, generation);
    assert_eq!(session.undo.len(), 1);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document.as_ref(), Some(&document));
    session.history(session.generation, true).unwrap();
    assert_eq!(session.document.as_ref(), Some(&after));
}
