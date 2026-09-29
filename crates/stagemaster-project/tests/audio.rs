use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
fn document() -> Document {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = json!([]);
    value["lighting"]["sequences"] = json!([]);
    Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}
fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(EditCommand::Audio {
        command: serde_json::from_value(command).unwrap(),
    })
}
fn with_audio() -> Document {
    let mut doc = document();
    edit(&mut doc,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"音乐.wav","extension":"wav","durationMs":10000}})).unwrap();
    doc
}
fn marker(index: u8, time: u64, scene: Option<&str>) -> Value {
    json!({"id":format!("a0000000-0000-4000-8000-{index:012}"),"name":format!("卡点 {index}"),"timeMs":time,"sceneId":scene})
}
#[test]
fn roundtrip_preserves_markers_capability_and_scene_references() {
    let mut doc = with_audio();
    let scene = doc.view().scenes[0].id.clone();
    edit(
        &mut doc,
        json!({"kind":"putMarker","marker":marker(2,5000,None)}),
    )
    .unwrap();
    edit(
        &mut doc,
        json!({"kind":"putMarker","marker":marker(1,1000,Some(&scene))}),
    )
    .unwrap();
    let track = doc.audio_timeline().unwrap();
    assert_eq!(track.markers[0].time_ms, 1000);
    assert!(track.scene_at(999).is_none());
    assert_eq!(
        track.scene_at(5000).unwrap().scene_id.as_deref(),
        Some(scene.as_str())
    );
    assert_eq!(doc, Document::decode(&doc.encode().unwrap()).unwrap());
    let before = doc.clone();
    assert!(
        doc.edit(serde_json::from_value(json!({"op":"removeScene","id":scene})).unwrap())
            .unwrap_err()
            .contains("卡点")
    );
    assert_eq!(doc, before);
    let mut old: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    old["requires"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["key"] != "media.audio-editing");
    assert!(
        Document::decode(&serde_json::to_vec(&old).unwrap())
            .unwrap_err()
            .contains("能力")
    );
}
#[test]
fn edits_reject_collisions_out_of_range_and_unknown_fields_atomically() {
    let mut doc = with_audio();
    edit(
        &mut doc,
        json!({"kind":"putMarker","marker":marker(1,5000,None)}),
    )
    .unwrap();
    let before = doc.clone();
    for command in [
        json!({"kind":"putMarker","marker":marker(2,5000,None)}),
        json!({"kind":"putMarker","marker":marker(2,10000,None)}),
        json!({"kind":"trim","inMs":2000,"outMs":6000}),
        json!({"kind":"trim","inMs":10000,"outMs":9000}),
    ] {
        assert!(edit(&mut doc, command).is_err());
        assert_eq!(doc, before);
    }
    edit(&mut doc, json!({"kind":"trim","inMs":2000,"outMs":9000})).unwrap();
    assert_eq!(doc.audio_timeline().unwrap().duration_ms(), 7000);
    let mut value: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    value["media"]["audioEditing"]["asset"]["filePath"] = json!("/private/song.wav");
    assert!(Document::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    edit(&mut doc, json!({"kind":"clear"})).unwrap();
    assert!(doc.audio_timeline().is_none());
    assert_eq!(doc.view().scenes.len(), before.view().scenes.len());
}
