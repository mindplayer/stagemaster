#[path = "support/controlled.rs"]
mod controlled;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use controlled::*;
use stagemaster_live_host::{
    Action,
    media::{ControlFailure, MediaCommand},
};
use stagemaster_runtime::Code;
use support::*;

#[test]
fn authority_revision_and_duplicate_receipts_precede_any_provider_intent() {
    let rig = rig(2000);
    assert_eq!(rig.port.control_state().unwrap(), None);
    let old = connect(&rig.host, 1, false, 60_000);
    let client = connect(&rig.host, 2, true, 60_000);
    let initial = client.acquired_state();
    let action = Action::RequestMedia {
        group: rig.port.initial().key,
        command: MediaCommand::Play,
    };
    let stale = old
        .submit(1, initial.revision, action.clone(), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap();
    assert_eq!(
        stale,
        Err(stagemaster_runtime_host::Error::Runtime(Code::Lease))
    );
    assert_eq!(rig.port.control_state().unwrap(), None);
    let wrong = client
        .submit(1, initial.revision - 1, action.clone(), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(wrong.result, Err(Code::Revision));
    assert_eq!(rig.port.control_state().unwrap(), None);
    let first = client
        .submit(2, wrong.state.revision, action.clone(), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(first.result, Ok(()));
    let intent = pending(&first.state);
    let duplicate = client
        .submit(2, wrong.state.revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(duplicate, first);
    assert_eq!(rig.port.control_state().unwrap().unwrap().request, intent);
    client.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    assert_eq!(rig.port.control_state().unwrap().unwrap().request, intent);
    assert_eq!(rig.port.control_state().unwrap().unwrap().result, None);
}

#[test]
fn newer_stop_invalidates_old_preparation_and_timeout_never_activates_it() {
    let rig = rig(1000);
    let client = connect(&rig.host, 1, false, 60_000);
    let started = request(&client, 1, &client.acquired_state(), MediaCommand::Play);
    let old = pending(&started);
    let prepared = rig
        .prepare
        .prepare(rig.port.initial().key, &rig.doc, 0, true, old.deadline_ms)
        .unwrap();
    let stopped = request(&client, 2, &started, MediaCommand::Stop);
    let stop = pending(&stopped);
    let (sample, map) = rig.sample(1, 0, true, std::time::Instant::now());
    assert!(matches!(
        rig.port.stage_requested(old.ticket, prepared, sample, map),
        Err(Code::State)
    ));
    assert_eq!(
        rig.port.complete_control(old.ticket, Ok(())),
        Err(Code::State)
    );
    rig.port.complete_control(stop.ticket, Ok(())).unwrap();
    let stopped = completed(&rig, stop);
    assert_eq!(
        stopped.media[0].unwrap().group.status,
        stagemaster_live::media::Status::Stopped
    );
    let seek = request(
        &client,
        3,
        &stopped,
        MediaCommand::Seek {
            position_ms: 100,
            playing: false,
        },
    );
    let seek = pending(&seek);
    let expired = until(&rig.host.observer(), |s| {
        s.state.media[0]
            .unwrap()
            .control
            .is_some_and(|c| c.result == Some(Err(ControlFailure::TimedOut)))
    });
    assert_eq!(
        expired.state.media[0].unwrap().control.unwrap().request,
        seek
    );
    assert_eq!(
        rig.port.complete_control(seek.ticket, Ok(())),
        Err(Code::State)
    );
    assert_eq!(
        expired.state.media[0].unwrap().group.status,
        stagemaster_live::media::Status::Stopped
    );
}

#[test]
fn unregistered_control_bad_position_and_legacy_bypass_are_rejected() {
    let ordinary = media_support::Rig::new();
    assert_eq!(ordinary.port.control_state(), Err(Code::State));
    let client = connect(&ordinary.host, 1, false, 60_000);
    let state = client.acquired_state();
    let result = client
        .submit(
            1,
            state.revision,
            Action::RequestMedia {
                group: ordinary.port.initial().key,
                command: MediaCommand::Play,
            },
            TTL,
        )
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(result.result, Err(Code::State));

    let rig = rig(2000);
    let client = connect(&rig.host, 2, false, 60_000);
    let mut state = client.acquired_state();
    for (serial, action) in [
        Action::RequestMedia {
            group: rig.port.initial().key,
            command: MediaCommand::Seek {
                position_ms: 1001,
                playing: true,
            },
        },
        Action::StopMedia {
            group: rig.port.initial().key,
        },
    ]
    .into_iter()
    .enumerate()
    {
        let result = client
            .submit(
                u64::try_from(serial + 1).unwrap(),
                state.revision,
                action,
                TTL,
            )
            .unwrap()
            .wait(WAIT)
            .unwrap()
            .unwrap();
        assert_eq!(result.result, Err(Code::State));
        state = result.state;
    }
    assert_eq!(rig.port.control_state().unwrap(), None);
    let (sample, map) = rig.sample(1, 0, false, std::time::Instant::now());
    let prepared = rig
        .prepare
        .prepare(rig.port.initial().key, &rig.doc, 0, false, 2000)
        .unwrap();
    assert!(matches!(
        rig.port.stage(prepared, sample, map),
        Err(Code::State)
    ));
}
