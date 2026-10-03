#[path = "support/media_host.rs"]
mod media_support;
mod support;
use media_support::Rig;
use stagemaster_live::{Command, media::Status};
use stagemaster_live_host::Action;
use stagemaster_runtime::Code;
use std::time::Instant;
use support::*;

#[test]
fn activation_receipt_retry_withdrawal_and_old_generation_keep_original_authority_rules() {
    let mut rig = Rig::new();
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let key = rig.port.initial().key;
    let (sample, map) = rig.sample(1, 0, true, Instant::now());
    let prepared = rig
        .prepare
        .prepare(key, &rig.doc, 0, true, state.observed_ms + 2000)
        .unwrap();
    let ticket = rig.port.stage(prepared, sample, map).unwrap();
    rig.reached(sample, &map);
    let action = Action::ActivateMedia { ticket };
    let receipt = client
        .submit(2, state.revision, action.clone(), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    receipt.result.unwrap();
    let active = receipt.state.media[0].unwrap().group.key;
    assert_ne!(active, key);
    assert_eq!(
        rig.port.reclaim(ticket, false).unwrap().result,
        Some(Ok(()))
    );
    let duplicate = client
        .submit(2, state.revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(duplicate, receipt); // Plan was already reclaimed; retry still uses historical receipt.
    let prepared = rig
        .prepare
        .prepare(active, &rig.doc, 80, true, receipt.state.observed_ms + 2000)
        .unwrap();
    let (sample, map) = rig.sample(2, 80, true, Instant::now());
    let cancelled = rig.port.stage(prepared, sample, map).unwrap();
    assert_eq!(rig.port.reclaim(cancelled, true).unwrap().result, None);
    let rejected = client
        .submit(
            3,
            receipt.state.revision,
            Action::ActivateMedia { ticket: cancelled },
            TTL,
        )
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(rejected.result, Err(Code::State));
    assert_eq!(rejected.state.media[0].unwrap().group.key, active);
    let state = send(
        &client,
        4,
        rejected.state.revision,
        Action::StopMedia { group: active },
    );
    assert_eq!(state.media[0].unwrap().group.status, Status::Stopped);
    assert_eq!(
        state.sources[1].unwrap().status,
        Some(stagemaster_runtime::Status::Running)
    );
    rig.reached(sample, &map);
    let serial = rig.port.publish(active, sample, map).unwrap();
    let after = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == serial)
    });
    assert_eq!(
        after.state.media[0].unwrap().observation.unwrap().result,
        Err(Code::State)
    );
    assert_eq!(after.state.media[0].unwrap().group.status, Status::Stopped);
    rig.host.shutdown(WAIT).unwrap();
}

#[test]
fn preparation_catches_up_off_worker_and_bad_samples_cannot_renew_or_retime_other_sources() {
    let mut rig = Rig::new();
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
        .prepare(initial, &rig.doc, 0, false, state.observed_ms + 2000)
        .unwrap();
    let prepared = prepared.advance_to(80, true).unwrap();
    let (sample, map) = rig.sample(1, 80, true, Instant::now());
    let ticket = rig.port.stage(prepared, sample, map).unwrap();
    rig.reached(sample, &map);
    let state = send(&client, 2, state.revision, Action::ActivateMedia { ticket });
    drop(rig.port.reclaim(ticket, false).unwrap());
    let active = state.media[0].unwrap().group.key;
    assert_eq!(state.media[0].unwrap().group.position_ms, 80);
    let composed = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.key == active
    });
    rig.assert_light(&composed.frame.unwrap().slots, [25_000, 30_000, 0]);
    assert!(
        rig.prepare
            .prepare(active, &rig.doc, 80, true, 2000)
            .unwrap()
            .advance_to(0, true)
            .is_err()
    );
    let (sample, map) = rig.sample(2, 80, false, Instant::now());
    rig.reached(sample, &map);
    let pause = rig.port.publish(active, sample, map).unwrap();
    let paused = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == pause)
    });
    assert_eq!(paused.state.media[0].unwrap().group.status, Status::Paused);
    let serial = rig.port.publish(active, sample, map).unwrap(); // A cached read is not a new sample.
    let rejected = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .observation
            .is_some_and(|r| r.serial == serial)
    });
    assert_eq!(
        rejected.state.media[0].unwrap().observation.unwrap().result,
        Err(Code::State)
    );
    assert_eq!(rejected.state.revision, state.revision); // Observation does not steal operator revision.
    let lost = until(&rig.host.observer(), |s| {
        s.state.media[0].unwrap().group.status == Status::Lost
    });
    let changed = until(&rig.host.observer(), |s| {
        s.frame.unwrap().slots[3] != lost.frame.unwrap().slots[3]
    });
    assert_eq!(changed.state.media[0].unwrap().group.position_ms, 80);
    assert_eq!(
        changed.state.sources[1].unwrap().status,
        Some(stagemaster_runtime::Status::Running)
    );
    rig.host.shutdown(WAIT).unwrap();
}
