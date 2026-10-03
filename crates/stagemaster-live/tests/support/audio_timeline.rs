use crate::{media_support::*, support::*};
use serde_json::json;
use stagemaster_live::{PlaybackSelection, Session};
use stagemaster_project::{Document, EditCommand};

pub fn document_with_audio() -> Document {
    let mut doc = document();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":1000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"第一段","sceneId":id(1),"startMs":0,"endMs":200,"fadeMs":0}),
        json!({"kind":"addLightingClip","name":"交叉段","sceneId":id(3),"startMs":200,"endMs":400,"fadeMs":100,"fadeMode":"dynamic"}),
        json!({"kind":"addLightingClip","name":"第三段","sceneId":id(2),"startMs":500,"endMs":900,"fadeMs":100}),
    ] {
        doc.edit(EditCommand::Audio {
            command: serde_json::from_value(command).unwrap(),
        })
        .unwrap();
    }
    doc
}
pub fn prepared(doc: &Document) -> Session {
    let mut sources = specs();
    sources[0].playback = Some(PlaybackSelection::AudioTimeline);
    Session::prepare_with_media(doc, [9; 16], &sources, &[group()], 1000).unwrap()
}
pub fn reference(doc: &Document, position: u64) -> Vec<u16> {
    let track = doc.audio_timeline().unwrap();
    let clip = track.lighting_at(position);
    let compiled = doc.compile_audio_segment(clip.map(|c| c.id)).unwrap();
    let mut player = compiled.playback.into_player().unwrap();
    player
        .advance(position - clip.map_or(0, |c| c.start_ms))
        .unwrap();
    player.values().to_vec()
}
