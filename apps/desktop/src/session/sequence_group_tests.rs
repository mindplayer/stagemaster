use super::*;
use crate::preview::{Command, Request};
use serde_json::{Value, json};
#[test]
fn group_history_is_atomic_and_running_plan_survives_edit_and_undo() {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let v = doc.view();
    let seq = &v.sequences[0];
    let mut session = Session::default();
    session.replace(doc.clone(), None);
    session
        .preview(Request::Load {
            generation: session.generation,
            sequence_id: seq.id.clone(),
        })
        .unwrap();
    let snapshot = serde_json::to_value(session.preview(Request::Snapshot).unwrap()).unwrap();
    let epoch = u32::try_from(snapshot["epoch"].as_u64().unwrap()).unwrap();
    session
        .preview(Request::Control {
            epoch,
            serial: 1,
            command: Command::Next,
        })
        .unwrap();
    session
        .preview(Request::Control {
            epoch,
            serial: 2,
            command: Command::Pause,
        })
        .unwrap();
    let loaded =
        serde_json::to_value(session.preview(Request::Snapshot).unwrap()).unwrap()["loaded"]
            .clone();
    let command = |kind: &str, ids: Vec<String>, before: Option<&str>| {
        let mut op = json!({"kind":kind});
        if kind != "remove" {
            op["beforeId"] = json!(before);
        }
        serde_json::from_value(json!({"op":"sequence","command":{"kind":"editSteps","id":seq.id,"stepIds":ids,"operation":op}})).unwrap()
    };
    let original_ids: Vec<_> = seq.steps.iter().map(|s| s.id.clone()).collect();
    let generation = session.generation;
    session
        .edit(generation, command("move", original_ids.clone(), None))
        .unwrap();
    assert_eq!(session.generation, generation);
    assert!(session.undo.is_empty());
    session
        .edit(
            session.generation,
            command("copy", original_ids.clone(), None),
        )
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after =
        serde_json::to_value(session.preview(Request::Snapshot).unwrap()).unwrap()["loaded"]
            .clone();
    assert_eq!(after["steps"], loaded["steps"]);
    assert_eq!(after["stepId"], loaded["stepId"]);
    assert_eq!(after["status"], "paused");
    assert_eq!(after["stale"], true);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document.as_ref(), Some(&doc));
    let before_generation = session.generation;
    assert!(
        session
            .edit(session.generation, command("remove", original_ids, None))
            .is_err()
    );
    assert_eq!(session.generation, before_generation);
    assert_eq!(session.redo.len(), 1);
    session.history(session.generation, true).unwrap();
    assert_eq!(
        session.document.as_ref().unwrap().view().sequences[0]
            .steps
            .len(),
        seq.steps.len() * 2
    );
}
