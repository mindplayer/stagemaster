#[path = "support/audio.rs"]
#[allow(dead_code)]
mod audio_support;
#[path = "support/controlled.rs"]
mod controlled;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use audio_support::{consume, prepared_audio};
use controlled::*;
use media_support::Rig;
use stagemaster_audio::PerformanceControl;
use stagemaster_live::{
    Command,
    media::{GroupKey, Sample, Status},
};
use stagemaster_live_host::media::MediaCommand;
use stagemaster_live_host::{State, media::ControlRequest};
use stagemaster_time::Mapping;
use std::sync::atomic::AtomicBool;
use support::*;

fn sample(rig: &Rig, audio: &PerformanceControl) -> (Sample, Mapping) {
    let snapshot = audio.snapshot().unwrap();
    let render = snapshot.render.unwrap();
    let position = snapshot.position.tick * 1000 / u64::from(render.sample_rate);
    let result = rig.sample(render.sequence, position, render.applied.playing, render.at);
    rig.reached(result.0, &result.1);
    result
}
fn publish(rig: &Rig, audio: &PerformanceControl, key: GroupKey) -> State {
    let (sample, map) = sample(rig, audio);
    let serial = rig.port.publish(key, sample, map).unwrap();
    let state = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == serial)
    })
    .state;
    assert_eq!(state.media[0].unwrap().observation.unwrap().result, Ok(()));
    state
}
fn activate(
    rig: &Rig,
    audio: &PerformanceControl,
    request: ControlRequest,
    key: GroupKey,
) -> State {
    let (sample, map) = sample(rig, audio);
    let prepared = rig
        .prepare
        .prepare(
            key,
            &rig.doc,
            sample.position_ms,
            sample.playing,
            request.deadline_ms,
        )
        .unwrap();
    let ticket = rig
        .port
        .stage_requested(request.ticket, prepared, sample, map)
        .unwrap();
    let state = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.key != key
    })
    .state;
    assert_eq!(
        rig.port.reclaim(ticket, false).unwrap().result,
        Some(Ok(()))
    );
    assert_eq!(state.media[0].unwrap().control.unwrap().result, None);
    state
}

#[test]
fn authorized_start_pause_seek_and_stop_wait_for_the_real_native_source() {
    let mut rig = rig(2000);
    let (_dir, audio) = prepared_audio();
    let (mut source, native) = audio.source(0, &AtomicBool::new(false)).unwrap();
    native.request_playback(false).unwrap();
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let state = request(&client, 2, &state, MediaCommand::Play);
    let play = pending(&state);
    assert!(native.snapshot().unwrap().render.is_none());
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    drop(client);
    consume(&mut source, 1); // Paused, zero samples: initial health without consuming material.
    let state = activate(&rig, &native, play, rig.port.initial().key);
    assert_eq!(state.media[0].unwrap().group.status, Status::Paused);
    assert!(native.snapshot().unwrap().consumption.is_none());
    assert!(state.owner.is_none());
    native.request_playback(true).unwrap();
    consume(&mut source, 10);
    let key = state.media[0].unwrap().group.key;
    publish(&rig, &native, key);
    rig.port.complete_control(play.ticket, Ok(())).unwrap();
    let state = completed(&rig, play);
    assert_eq!(state.media[0].unwrap().group.position_ms, 10);

    let client = connect(&rig.host, 2, false, 60_000);
    let state = request(&client, 1, &client.acquired_state(), MediaCommand::Pause);
    let pause = pending(&state);
    assert_eq!(state.media[0].unwrap().group.status, Status::Following);
    native.request_playback(false).unwrap();
    assert!(native.snapshot().unwrap().render.unwrap().applied.playing);
    consume(&mut source, 1);
    let state = publish(&rig, &native, key);
    assert_eq!(state.media[0].unwrap().group.position_ms, 10);
    rig.port.complete_control(pause.ticket, Ok(())).unwrap();
    let paused = completed(&rig, pause);
    assert_eq!(paused.media[0].unwrap().group.status, Status::Paused);

    let state = request(
        &client,
        2,
        &paused,
        MediaCommand::Seek {
            position_ms: 100,
            playing: false,
        },
    );
    let seek = pending(&state);
    let (mut replacement, next) = audio.source(100, &AtomicBool::new(false)).unwrap();
    next.request_playback(false).unwrap();
    native.cancel();
    drop(source);
    consume(&mut replacement, 1);
    let state = activate(&rig, &next, seek, key);
    assert_eq!(state.media[0].unwrap().group.position_ms, 100);
    rig.port.complete_control(seek.ticket, Ok(())).unwrap();
    let positioned = completed(&rig, seek);
    assert_ne!(positioned.media[0].unwrap().group.key, key);
    let frame = until(&rig.host.observer(), |s| s.frame.is_some())
        .frame
        .unwrap();
    assert_eq!(frame.slots[1], 117); // Actual composed red at the authored crossfade endpoint.
    assert_eq!(frame.slots[3], 195); // Autonomous blue continues through all media operations.

    let state = request(&client, 3, &positioned, MediaCommand::Stop);
    let stop = pending(&state);
    next.cancel();
    assert_eq!(replacement.next(), None);
    rig.port.complete_control(stop.ticket, Ok(())).unwrap();
    let stopped = completed(&rig, stop);
    assert_eq!(stopped.media[0].unwrap().group.status, Status::Stopped);
    assert_eq!(
        stopped.sources[1].unwrap().status,
        Some(stagemaster_runtime::Status::Running)
    );
    rig.host.shutdown(WAIT).unwrap();
    assert_eq!(
        rig.port.control_state(),
        Err(stagemaster_runtime::Code::State)
    );
}
