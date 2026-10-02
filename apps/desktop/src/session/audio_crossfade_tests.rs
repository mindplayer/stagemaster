use super::*;
use serde_json::json;
#[test]
fn crossfade_group_and_split_are_single_history_entries_and_stale_edits_are_rejected() {
    let mut doc = Document::new("动态交叉历史").unwrap();
    let view = doc.view();
    for op in [
        json!({"op":"addFixture","name":"灯","profileId":view.profiles[0].id,"domainId":view.domains[0].id,"universe":1,"address":1}),
        json!({"op":"addScene","name":"场景"}),
    ] {
        doc.edit(serde_json::from_value(op).unwrap()).unwrap();
    }
    let scene = doc.view().scenes[0].id.clone();
    let edit = |c| serde_json::from_value(json!({"op":"audio","command":c})).unwrap();
    for c in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":5000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"段","sceneId":scene,"startMs":0,"endMs":5000,"fadeMs":500}),
    ] {
        doc.edit(edit(c)).unwrap();
    }
    let id = doc.audio_timeline().unwrap().lighting_clips.unwrap()[0]
        .id
        .clone();
    let mut session = Session::default();
    session.replace(doc.clone(), None);
    let generation = session.generation;
    session.edit(generation,edit(json!({"kind":"editLightingClips","ids":[id],"action":{"kind":"fade","fadeMs":750,"fadeMode":"dynamic"}}))).unwrap();
    assert_eq!(session.undo.len(), 1);
    let mode = session.document.clone();
    assert!(
        session
            .edit(
                generation,
                edit(json!({"kind":"splitLightingClip","id":id,"timeMs":333}))
            )
            .is_err()
    );
    assert_eq!(session.document, mode);
    session
        .edit(
            session.generation,
            edit(json!({"kind":"splitLightingClip","id":id,"timeMs":333})),
        )
        .unwrap();
    assert_eq!(session.undo.len(), 2);
    let split = session.document.clone();
    let serialized = split.as_ref().unwrap().encode().unwrap();
    assert_eq!(
        Document::decode(&serialized).unwrap(),
        split.clone().unwrap()
    );
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document, mode);
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document, Some(doc));
    session.history(session.generation, true).unwrap();
    session.history(session.generation, true).unwrap();
    assert_eq!(session.document, split);
}
