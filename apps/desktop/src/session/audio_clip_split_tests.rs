use super::*;
use serde_json::json;
#[test]
fn split_is_one_history_entry_with_atomic_failure_and_reset_noop() {
    let mut d = Document::new("分割历史").unwrap();
    let v = d.view();
    d.edit(serde_json::from_value(json!({"op":"addFixture","name":"灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
    d.edit(serde_json::from_value(json!({"op":"addScene","name":"场景"})).unwrap())
        .unwrap();
    let scene = d.view().scenes[0].id.clone();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":5000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"段","sceneId":scene,"startMs":0,"endMs":5000,"fadeMs":500}),
    ] {
        d.edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
    let id = d.audio_timeline().unwrap().lighting_clips.unwrap()[0]
        .id
        .clone();
    let command =
        |command| serde_json::from_value(json!({"op":"audio","command":command})).unwrap();
    let mut session = Session::default();
    session.replace(d.clone(), None);
    let generation = session.generation;
    session
        .edit(
            generation,
            command(json!({"kind":"resetLightingClipEffectOffset","id":id})),
        )
        .unwrap();
    assert_eq!(session.generation, generation);
    assert!(session.undo.is_empty());
    session
        .edit(
            generation,
            command(json!({"kind":"splitLightingClip","id":id,"timeMs":1333})),
        )
        .unwrap();
    assert_eq!(session.undo.len(), 1);
    let after = session.document.clone();
    session.history(session.generation, false).unwrap();
    assert_eq!(session.document, Some(d));
    let generation = session.generation;
    assert!(
        session
            .edit(
                generation,
                command(json!({"kind":"splitLightingClip","id":id,"timeMs":499}))
            )
            .is_err()
    );
    assert_eq!(session.generation, generation);
    assert_eq!(session.redo.len(), 1);
    session.history(generation, true).unwrap();
    assert_eq!(session.document, after);
}
