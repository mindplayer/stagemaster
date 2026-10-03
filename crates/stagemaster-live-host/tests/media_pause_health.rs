#[path = "support/audio.rs"]
mod audio_support;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use audio_support::{audio, consume};
use media_support::Rig;
use stagemaster_audio::{PerformanceControl, PerformanceSource};
use stagemaster_live::{
    Command,
    media::{GroupKey, Sample, Status},
};
use stagemaster_live_host::{Action, State};
use stagemaster_time::Mapping;
use std::time::Duration;
use support::*;

fn observation(rig: &Rig, audio: &PerformanceControl) -> (Sample, Mapping) {
    let snapshot = audio.try_snapshot().unwrap();
    assert!(snapshot.problem.is_none() && !snapshot.stopped);
    let render = snapshot.render.unwrap();
    assert_eq!(render.instance, audio.instance());
    rig.sample(
        render.sequence,
        snapshot.position.tick * 1000 / u64::from(render.sample_rate),
        render.applied.playing,
        render.at,
    )
}
fn publish(rig: &Rig, key: GroupKey, sample: Sample, map: Mapping) -> State {
    rig.reached(sample, &map);
    let serial = rig.port.publish(key, sample, map).unwrap();
    let accepted = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == serial)
    });
    accepted.state.media[0]
        .unwrap()
        .observation
        .unwrap()
        .result
        .unwrap();
    accepted.state
}

fn start(
    rig: &mut Rig,
    source: &mut PerformanceSource,
    audio: &PerformanceControl,
) -> (GroupKey, Sample) {
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let prepared = rig
        .prepare
        .prepare(
            rig.port.initial().key,
            &rig.doc,
            0,
            true,
            state.observed_ms + 2000,
        )
        .unwrap();
    consume(source, 10);
    let (sample, map) = observation(rig, audio);
    rig.reached(sample, &map);
    let ticket = rig
        .port
        .stage(prepared.advance_to(10, true).unwrap(), sample, map)
        .unwrap();
    let state = send(&client, 2, state.revision, Action::ActivateMedia { ticket });
    drop(rig.port.reclaim(ticket, false).unwrap());
    let key = state.media[0].unwrap().group.key;
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    drop(client);

    (key, sample)
}

#[test]
fn callback_confirmed_pause_stays_healthy_but_stopped_pulls_cannot_renew_the_group() {
    let mut rig = Rig::new();
    let (_dir, mut source, audio) = audio();
    let (key, sample) = start(&mut rig, &mut source, &audio);

    let before = audio.snapshot().unwrap();
    audio.request_playback(false).unwrap();
    // Host intent alone cannot change the confirmed state or generate fresh time.
    assert_eq!(observation(&rig, &audio).0, sample);
    consume(&mut source, 1);
    let (paused, map) = observation(&rig, &audio);
    let first = publish(&rig, key, paused, map);
    assert_eq!(first.media[0].unwrap().group.status, Status::Paused);
    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(80));
        consume(&mut source, 1); // Real complete silent frames; no material consumption.
        let snapshot = audio.snapshot().unwrap();
        assert_eq!(snapshot.consumption, before.consumption);
        assert_eq!(snapshot.position, before.position);
        let (sample, map) = observation(&rig, &audio);
        let state = publish(&rig, key, sample, map);
        assert_eq!(state.media[0].unwrap().group.status, Status::Paused);
        assert_eq!(state.media[0].unwrap().group.position_ms, 10);
        assert!(state.owner.is_none());
    }
    let healthy = until(&rig.host.observer(), |s| {
        s.state.observed_ms >= first.observed_ms + 600
    });
    assert_eq!(healthy.state.media[0].unwrap().group.status, Status::Paused);

    // Resume advances from exactly the frozen material position without replacing the instance.
    audio.request_playback(true).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    consume(&mut source, 100);
    let resumed = audio.snapshot().unwrap().consumption.unwrap();
    assert_eq!(resumed.instance, before.consumption.unwrap().instance);
    assert_eq!(resumed.frames, before.consumption.unwrap().frames + 800);
    let (sample, map) = observation(&rig, &audio);
    let state = publish(&rig, key, sample, map);
    assert_eq!(state.media[0].unwrap().group.status, Status::Following);
    assert_eq!(state.media[0].unwrap().group.position_ms, 110);
    audio.request_playback(false).unwrap();
    consume(&mut source, 1);
    let (cached, map) = observation(&rig, &audio);
    publish(&rig, key, cached, map);

    // A live control handle and repeated reads do not mean the audio consumer is alive.
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(observation(&rig, &audio).0, cached);
    let (_, old_map) = observation(&rig, &audio);
    let serial = rig.port.publish(key, cached, old_map).unwrap();
    let rejected = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == serial)
    });
    assert!(
        rejected.state.media[0]
            .unwrap()
            .observation
            .unwrap()
            .result
            .is_err()
    );
    let lost = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.status == Status::Lost
    });
    assert_eq!(lost.state.media[0].unwrap().group.position_ms, 110);
    assert_eq!(
        audio.snapshot().unwrap().render.unwrap().sequence,
        cached.sequence
    );
    rig.assert_light(&lost.frame.unwrap().slots, [25_000, 30_000, 0]);
    let progressed = until(&rig.host.observer(), |s| {
        s.frame.unwrap().slots[3] != lost.frame.unwrap().slots[3]
    });
    assert!(progressed.state.owner.is_none());
    assert_eq!(
        progressed.state.sources[1].unwrap().status,
        Some(stagemaster_runtime::Status::Running)
    );
    rig.host.shutdown(WAIT).unwrap();
}
