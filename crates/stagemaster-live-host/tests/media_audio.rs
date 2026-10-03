#[path = "support/media_host.rs"]
mod media_support;
mod support;
use media_support::Rig;
#[path = "support/audio.rs"]
mod audio_support;
use audio_support::{audio, consume};
use stagemaster_live::{Command, media::Status};
use stagemaster_live_host::Action;
use std::time::Duration;
use support::*;

#[test]
fn actual_decoded_audio_keeps_driving_media_after_operator_disconnect_then_times_out_independently()
{
    let mut rig = Rig::new();
    let (_dir, mut source, audio) = audio();
    assert!(audio.snapshot().unwrap().consumption.is_none());
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let initial = rig.port.initial().key;
    let prepared = rig
        .prepare
        .prepare(initial, &rig.doc, 0, true, state.observed_ms + 2000)
        .unwrap();
    consume(&mut source, 10);
    let observed = audio.snapshot().unwrap();
    let consumed = observed.consumption.unwrap();
    assert_eq!(consumed.frames, 80);
    let position = observed.position.tick * 1000 / u64::from(consumed.sample_rate);
    let (sample, map) = rig.sample(consumed.frames, position, true, consumed.at);
    let original = sample.at;
    let prepared = prepared.advance_to(position, true).unwrap();
    rig.reached(sample, &map);
    assert_eq!(rig.clock.map(consumed.at).unwrap().0, original); // Reading it later did not freshen it.
    let ticket = rig.port.stage(prepared, sample, map).unwrap();
    let state = send(&client, 2, state.revision, Action::ActivateMedia { ticket });
    drop(rig.port.reclaim(ticket, false).unwrap());
    let key = state.media[0].unwrap().group.key;
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    drop(client);
    let before = until(&rig.host.observer(), |s| s.state.owner.is_none());
    for _ in 0..5 {
        std::thread::sleep(Duration::from_millis(20));
        consume(&mut source, 20);
        let observed = audio.snapshot().unwrap();
        assert!(observed.problem.is_none());
        let consumed = observed.consumption.unwrap();
        let position = observed.position.tick * 1000 / u64::from(consumed.sample_rate);
        let (sample, map) = rig.sample(consumed.frames, position, true, consumed.at);
        rig.reached(sample, &map);
        let serial = rig.port.publish(key, sample, map).unwrap();
        let accepted = until(&rig.host.observer(), |s| {
            s.state.media[0]
                .unwrap()
                .observation
                .is_some_and(|r| r.serial == serial)
        });
        let group = accepted.state.media[0].unwrap();
        group.observation.unwrap().result.unwrap();
        assert_eq!(group.group.position_ms, position);
        assert!(accepted.state.owner.is_none());
        assert_eq!(
            accepted.state.sources[1].unwrap().status,
            Some(stagemaster_runtime::Status::Running)
        );
    }
    let lost = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.status == Status::Lost
    });
    assert_eq!(lost.state.media[0].unwrap().group.position_ms, 110);
    rig.assert_light(&lost.frame.unwrap().slots, [25_000, 30_000, 0]);
    assert!(lost.frame.unwrap().info.sequence > before.frame.unwrap().info.sequence);
    let progressed = until(&rig.host.observer(), |s| {
        s.frame.unwrap().slots[3] != lost.frame.unwrap().slots[3]
    });
    assert_eq!(progressed.state.media[0].unwrap().group.position_ms, 110);
    assert!(progressed.state.owner.is_none());
    rig.host.shutdown(WAIT).unwrap();
}
