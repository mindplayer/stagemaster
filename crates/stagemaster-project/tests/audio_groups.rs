use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Audio {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn id(index: usize) -> String {
    format!("a0000000-0000-4000-8000-{index:012}")
}
fn document() -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    edit(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"音乐.wav","extension":"wav","durationMs":10000}})).unwrap();
    let scene = doc.view().scenes[0].id.clone();
    for (index, time, lighting, fade) in [
        (1, 1000, true, 500),
        (2, 2000, false, 0),
        (3, 3000, true, 250),
        (4, 7000, true, 0),
    ] {
        edit(&mut doc,json!({"kind":"putMarker","marker":{"id":id(index),"name":format!("卡点{index}"),"timeMs":time,"sceneId":if lighting {Some(&scene)} else {None},"fadeMs":fade}})).unwrap();
    }
    doc
}
fn group(doc: &mut Document, ids: &[String], action: Value) -> Result<(), String> {
    let mut command = json!({"kind":"editMarkers","ids":ids});
    command["action"] = action;
    edit(doc, command)
}
#[test]
fn unordered_selection_moves_with_original_identity_and_relative_timing() {
    let mut doc = document();
    let before = doc.audio_timeline().unwrap();
    group(
        &mut doc,
        &[id(2), id(1)],
        json!({"kind":"move","destinationMs":4000}),
    )
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    for (i, t) in [(1, 4000), (2, 5000), (3, 3000), (4, 7000)] {
        let marker = track.markers.iter().find(|m| m.id == id(i)).unwrap();
        let old = before.markers.iter().find(|m| m.id == id(i)).unwrap();
        assert_eq!(marker.time_ms, t);
        assert_eq!(marker.name, old.name);
        assert_eq!(marker.scene_id, old.scene_id);
        assert_eq!(marker.fade_ms, old.fade_ms);
    }
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
}
#[test]
fn copy_creates_new_identities_preserves_source_and_can_be_deleted_independently() {
    let mut doc = document();
    let before = doc.audio_timeline().unwrap();
    group(
        &mut doc,
        &[id(2), id(1)],
        json!({"kind":"copy","destinationMs":5000}),
    )
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    assert_eq!(track.markers.len(), 6);
    for old in &before.markers {
        assert!(track.markers.contains(old));
    }
    let copies: Vec<_> = track
        .markers
        .iter()
        .filter(|m| !before.markers.iter().any(|old| old.id == m.id))
        .collect();
    assert_eq!(copies[0].time_ms, 5000);
    assert_eq!(copies[1].time_ms, 6000);
    assert_eq!(copies[0].scene_id, before.markers[0].scene_id);
    assert_eq!(copies[0].fade_ms, 500);
    let ids: Vec<_> = copies.iter().map(|m| m.id.clone()).collect();
    group(&mut doc, &ids, json!({"kind":"remove"})).unwrap();
    assert_eq!(doc.audio_timeline().unwrap(), before);
}
#[test]
fn collisions_range_unknown_selection_and_neighbor_fades_reject_atomically() {
    let mut doc = document();
    for (ids, action) in [
        (
            vec![id(1), id(2)],
            json!({"kind":"move","destinationMs":3000}),
        ),
        (
            vec![id(1), id(2)],
            json!({"kind":"copy","destinationMs":9800}),
        ),
        (vec![id(1)], json!({"kind":"copy","destinationMs":1200})),
        (vec![id(3)], json!({"kind":"move","destinationMs":6800})),
        (vec![id(1)], json!({"kind":"move","destinationMs":u64::MAX})),
        (vec![id(1), id(1)], json!({"kind":"remove"})),
        (vec![id(99)], json!({"kind":"remove"})),
        (vec![], json!({"kind":"remove"})),
    ] {
        let before = doc.clone();
        assert!(group(&mut doc, &ids, action).is_err());
        assert_eq!(doc, before);
    }
    // A pure beat can sit inside a transition without becoming a lighting boundary.
    group(
        &mut doc,
        &[id(2)],
        json!({"kind":"copy","destinationMs":1200}),
    )
    .unwrap();
}
#[test]
fn all_512_markers_can_be_removed_in_one_edit_and_copy_never_exceeds_budget() {
    let doc = document();
    let mut root: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    root["media"]["audioEditing"]["markers"] = json!(
        (0..512)
            .map(|i| json!({"id":id(i),"name":format!("拍点{i}"),"timeMs":i*10,"sceneId":null}))
            .collect::<Vec<_>>()
    );
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let before = doc.clone();
    assert!(
        group(
            &mut doc,
            &[id(0)],
            json!({"kind":"copy","destinationMs":9000})
        )
        .unwrap_err()
        .contains("512")
    );
    assert_eq!(doc, before);
    group(
        &mut doc,
        &(0..512).map(id).collect::<Vec<_>>(),
        json!({"kind":"remove"}),
    )
    .unwrap();
    assert!(doc.audio_timeline().unwrap().markers.is_empty());
    assert_eq!(doc.view().scenes.len(), before.view().scenes.len());
}
