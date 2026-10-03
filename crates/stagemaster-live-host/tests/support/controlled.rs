#![allow(dead_code)]
use crate::{
    media_support::Rig,
    support::{fixtures::*, *},
};
use serde_json::json;
use stagemaster_live::PlaybackSelection;
use stagemaster_live_host::{
    Action, Live, State,
    media::{ControlRequest, MediaCommand},
};
use stagemaster_runtime_host::Client;

pub fn rig(timeout_ms: u64) -> Rig {
    let mut raw = fixture(0);
    raw["lighting"]["scenes"][3]["assignments"] = json!([set("blue", 50_000)]);
    let mut doc = decode(&raw);
    for command in [
        json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":1000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"第一段","sceneId":id(1),"startMs":0,"endMs":50,"fadeMs":0}),
        json!({"kind":"addLightingClip","name":"交叉","sceneId":id(3),"startMs":50,"endMs":500,"fadeMs":50,"fadeMode":"dynamic"}),
    ] {
        doc.edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
    let mut sources = specs();
    sources[0].playback = Some(PlaybackSelection::AudioTimeline);
    Rig::with_controls(doc, &sources, timeout_ms)
}
pub fn request(client: &Client<Live>, serial: u64, state: &State, command: MediaCommand) -> State {
    send(
        client,
        serial,
        state.revision,
        Action::RequestMedia {
            group: state.media[0].unwrap().group.key,
            command,
        },
    )
}
pub fn pending(state: &State) -> ControlRequest {
    let state = state.media[0].unwrap().control.unwrap();
    assert!(state.result.is_none());
    state.request
}
pub fn completed(rig: &Rig, request: ControlRequest) -> State {
    let state = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .control
            .is_some_and(|c| c.request == request && c.result.is_some())
    })
    .state;
    assert_eq!(
        state.media[0].unwrap().control.unwrap().result,
        Some(Ok(()))
    );
    state
}
