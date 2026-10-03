#[path = "support/audio.rs"]
#[allow(dead_code)]
mod audio_support;
#[path = "support/controlled.rs"]
mod controlled;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use controlled::*;
use media_support::Rig;
use rodio::mixer::MixerSource;
use stagemaster_audio::{OutputBinding, Transport};
use stagemaster_live::media::{GroupKey, Sample, Status};
use stagemaster_live_host::{
    State,
    media::{ControlRequest, MediaCommand},
};
use stagemaster_time::Mapping;
use std::sync::atomic::AtomicBool;
use support::*;

fn pull(mixer: &mut MixerSource, ms: usize) -> bool {
    let mut heard = false;
    for sample in mixer.by_ref().take(ms * 8 * 2) {
        heard |= sample != 0.0;
    }
    heard
}
fn observation(rig: &Rig, transport: &Transport) -> (Sample, Mapping) {
    let native = transport.performance_observation().unwrap().unwrap();
    let render = native.snapshot.render.unwrap();
    assert_eq!(render.applied, native.request);
    let millis = native.snapshot.position.tick * 1000 / u64::from(render.sample_rate);
    let (sample, map) = rig.sample(render.sequence, millis, render.applied.playing, render.at);
    rig.reached(sample, &map);
    (sample, map)
}
fn publish(rig: &Rig, transport: &Transport, key: GroupKey) -> State {
    let (sample, map) = observation(rig, transport);
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
fn activate(rig: &Rig, transport: &Transport, request: ControlRequest, key: GroupKey) -> State {
    let (sample, map) = observation(rig, transport);
    assert!(!sample.playing);
    let ready = rig
        .prepare
        .prepare(
            key,
            &rig.doc,
            sample.position_ms,
            false,
            request.deadline_ms,
        )
        .unwrap();
    let activation = rig
        .port
        .stage_requested(request.ticket, ready, sample, map)
        .unwrap();
    let state = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.key != key
    })
    .state;
    assert_eq!(
        rig.port.reclaim(activation, false).unwrap().result,
        Some(Ok(()))
    );
    assert_eq!(state.media[0].unwrap().control.unwrap().result, None);
    state
}
fn playing_rig(transport: &mut Transport, mixer: &mut MixerSource) -> (Rig, State) {
    let rig = rig(2000);
    let client = connect(&rig.host, 1, false, 60_000);
    let state = request(&client, 1, &client.acquired_state(), MediaCommand::Play);
    let intent = pending(&state);
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    drop(client);
    transport.prime_performance().unwrap();
    assert!(!pull(mixer, 20));
    let state = activate(&rig, transport, intent, rig.port.initial().key);
    assert!(state.owner.is_none());
    assert_eq!(state.media[0].unwrap().group.status, Status::Paused);
    transport.play().unwrap();
    assert_eq!(rig.port.control_state().unwrap().unwrap().result, None);
    assert!(pull(mixer, 20));
    publish(&rig, transport, state.media[0].unwrap().group.key);
    rig.port.complete_control(intent.ticket, Ok(())).unwrap();
    let state = completed(&rig, intent);
    assert!(state.media[0].unwrap().group.position_ms > 0);
    (rig, state)
}

#[test]
fn resident_transport_coordinates_native_player_and_host_after_operator_exit() {
    let (dir, _) = audio_support::prepared_audio();
    let (output, mut mixer) = rodio::mixer::mixer(2.try_into().unwrap(), 8000.try_into().unwrap());
    let mut transport = Transport::with_output(OutputBinding::new(output));
    let ready = transport
        .load_performance_request(dir.path().join("consumed.wav"), 0, 1000, None)
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_load(ready).unwrap();
    let (mut rig, state) = playing_rig(&mut transport, &mut mixer);
    let key = state.media[0].unwrap().group.key;
    let client = connect(&rig.host, 2, false, 60_000);
    let state = request(&client, 1, &client.acquired_state(), MediaCommand::Pause);
    let pause = pending(&state);
    transport.pause();
    pull(&mut mixer, 10);
    let paused = publish(&rig, &transport, key);
    rig.port.complete_control(pause.ticket, Ok(())).unwrap();
    let state = completed(&rig, pause);
    assert_eq!(
        state.media[0].unwrap().group.position_ms,
        paused.media[0].unwrap().group.position_ms
    );
    assert_eq!(state.media[0].unwrap().group.status, Status::Paused);
    let state = request(
        &client,
        2,
        &state,
        MediaCommand::Seek {
            position_ms: 100,
            playing: false,
        },
    );
    let seek = pending(&state);
    let prepared = transport
        .seek_preparation(100, false)
        .unwrap()
        .unwrap()
        .prepare(&AtomicBool::new(false))
        .unwrap();
    transport.apply_seek(prepared).unwrap();
    assert!(
        transport
            .performance_observation()
            .unwrap()
            .unwrap()
            .snapshot
            .render
            .is_none()
    );
    transport.prime_performance().unwrap();
    assert!(!pull(&mut mixer, 20));
    activate(&rig, &transport, seek, key);
    rig.port.complete_control(seek.ticket, Ok(())).unwrap();
    let state = completed(&rig, seek);
    assert_eq!(state.media[0].unwrap().group.position_ms, 100);
    let frame = until(&rig.host.observer(), |s| s.frame.is_some())
        .frame
        .unwrap();
    assert_eq!(frame.slots[1], 117); // Real authored red = 30000 at the fade endpoint, encoded to 8-bit.
    let state = request(&client, 3, &state, MediaCommand::Stop);
    let stop = pending(&state);
    transport.stop();
    assert!(transport.performance_observation().unwrap().is_none());
    assert!(!pull(&mut mixer, 20));
    rig.port.complete_control(stop.ticket, Ok(())).unwrap();
    assert_eq!(
        completed(&rig, stop).media[0].unwrap().group.status,
        Status::Stopped
    );
    rig.host.shutdown(WAIT).unwrap();
}
