use super::*;
use crate::preview::Request;
use serde_json::{Value, json};
#[test]
fn combined_timing_and_script_is_atomic_one_history_and_preserves_loaded_notes() {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let view = doc.view();
    let sequence = &view.sequences[0];
    let ids: Vec<_> = sequence.steps.iter().map(|s| s.id.clone()).collect();
    let command = |notes: &str| {
        serde_json::from_value::<EditCommand>(json!({"op":"batch","commands":[
            {"op":"sequence","command":{"kind":"editSteps","id":sequence.id,"stepIds":ids,"operation":{"kind":"timing","patch":{"fadeMs":1005}}}},
            {"op":"sequence","command":{"kind":"editSteps","id":sequence.id,"stepIds":ids,"operation":{"kind":"script","patch":{"section":"第二幕","notes":notes}}}}
        ]})).unwrap()
    };
    let mut session = Session::default();
    session.replace(doc.clone(), None);
    session
        .preview(Request::Load {
            generation: session.generation,
            sequence_id: sequence.id.clone(),
        })
        .unwrap();
    let loaded =
        serde_json::to_value(session.preview(Request::Snapshot).unwrap()).unwrap()["loaded"]
            .clone();
    session
        .edit(session.generation, command("等演员就位"))
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let revised = session.document.clone();
    for step in &revised.as_ref().unwrap().view().sequences[0].steps {
        assert_eq!(step.fade_ms, 1005);
        assert_eq!(step.script.as_ref().unwrap().section, "第二幕");
    }
    let runtime =
        serde_json::to_value(session.preview(Request::Snapshot).unwrap()).unwrap()["loaded"]
            .clone();
    assert_eq!(runtime["steps"], loaded["steps"]);
    assert_eq!(runtime["stale"], true);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document.as_ref(), Some(&doc));
    let generation = session.generation;
    assert!(
        session
            .edit(generation, command(&"字".repeat(4097)))
            .is_err()
    );
    assert_eq!(session.document.as_ref(), Some(&doc));
    assert_eq!(session.generation, generation);
    assert_eq!(session.redo.len(), 1);
    session.history(generation, true).unwrap();
    assert_eq!(session.document, revised);
    let generation = session.generation;
    session.edit(generation, command("等演员就位")).unwrap();
    assert_eq!(session.generation, generation);
    assert_eq!(session.undo.len(), 1);
}
