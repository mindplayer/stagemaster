use super::*;
use serde_json::json;
#[test]
fn clip_group_is_one_history_node_and_failure_preserves_redo() {
    let mut doc = Document::new("片段组历史").unwrap();
    let v = doc.view();
    doc.edit(serde_json::from_value(json!({"op":"addFixture","name":"灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
    doc.edit(serde_json::from_value(json!({"op":"addScene","name":"场景"})).unwrap())
        .unwrap();
    let scene = doc.view().scenes[0].id.clone();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":10000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"片段甲","sceneId":scene,"startMs":1000,"endMs":2000,"fadeMs":200}),
        json!({"kind":"addLightingClip","name":"片段乙","sceneId":scene,"startMs":3000,"endMs":4000,"fadeMs":0}),
    ] {
        doc.edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
    let ids: Vec<_> = doc
        .audio_timeline()
        .unwrap()
        .lighting_clips
        .unwrap()
        .iter()
        .map(|c| c.id.clone())
        .collect();
    let mut session = Session::default();
    session.replace(doc.clone(), None);
    let command = |kind: &str, time: u64| {
        serde_json::from_value(json!({"op":"audio","command":{"kind":"editLightingClips","ids":ids,"action":{"kind":kind,"destinationMs":time}}})).unwrap()
    };
    let generation = session.generation;
    session.edit(generation, command("move", 1000)).unwrap();
    assert_eq!(session.generation, generation);
    assert!(session.undo.is_empty());
    session
        .edit(session.generation, command("copy", 5000))
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after = session.document.clone();
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document.as_ref(), Some(&doc));
    let generation = session.generation;
    assert!(session.edit(generation, command("copy", 2500)).is_err());
    assert_eq!(session.generation, generation);
    assert_eq!(session.redo.len(), 1);
    session.history(generation, true).unwrap();
    assert_eq!(session.document, after);
}
