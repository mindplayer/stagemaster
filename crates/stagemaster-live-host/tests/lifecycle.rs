mod support;
use stagemaster_live::{Change, Command};
use stagemaster_live_host::{Action, Patch};
use stagemaster_runtime::{Code, Status};
use stagemaster_runtime_host::{Error, Phase};
use support::*;

#[test]
fn dropped_client_and_expired_lease_leave_dynamic_sources_running_in_the_same_worker() {
    let (mut host, keys) = start();
    let client = connect(&host, 1, false, 500);
    send(
        &client,
        1,
        client.acquired_state().revision,
        control(keys[3], Command::Execute(0)),
    );
    let observer = host.observer();
    let before = until(&observer, |s| s.frame.is_some());
    drop(client);
    let after = until(&observer, |s| {
        s.state.owner.is_none() && s.frame.unwrap().slots[0] != before.frame.unwrap().slots[0]
    });
    assert!(after.cycles > before.cycles);
    assert_eq!(
        after.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    let client = connect(&host, 2, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(keys[3], Command::Stop),
    );
    assert_eq!(state.sources[3].unwrap().status, Some(Status::Idle));
    assert_eq!(host.shutdown(WAIT).unwrap(), Phase::Stopped);
    assert!(observer.read().unwrap().snapshot.is_none());
}
#[test]
fn identical_retries_return_old_receipts_but_takeover_and_conflicting_retries_revoke_input() {
    let (mut host, keys) = start();
    let first = connect(&host, 1, false, 60_000);
    let revision = first.acquired_state().revision;
    let action = control(keys[3], Command::Execute(0));
    let receipt = first
        .submit(1, revision, action.clone(), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    receipt.result.unwrap();
    until(&host.observer(), |s| {
        s.state.observed_ms > receipt.state.observed_ms
    });
    let repeated = first
        .submit(1, revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(repeated, receipt);
    assert!(matches!(
        host.connect(grant(2, 60_000), false, TTL)
            .unwrap()
            .wait(WAIT)
            .unwrap(),
        Err(Error::Runtime(Code::Busy))
    ));
    let second = connect(&host, 2, true, 60_000);
    assert_eq!(
        first.release(TTL).unwrap().wait(WAIT).unwrap(),
        Err(Error::Runtime(Code::Lease))
    );
    assert!(matches!(
        first
            .submit(
                2,
                receipt.state.revision,
                control(keys[3], Command::Stop),
                TTL
            )
            .unwrap()
            .wait(WAIT)
            .unwrap(),
        Err(Error::Runtime(Code::Lease))
    ));
    let r = second.acquired_state().revision;
    let patch = Action::Patch {
        source: keys[2],
        patch: Patch::new(&[Change {
            attribute: 1,
            value: Some(60_000),
        }])
        .unwrap(),
    };
    let state = send(&second, 1, r, patch);
    assert!(matches!(
        second
            .submit(
                1,
                r,
                Action::Level {
                    source: keys[2],
                    level: 0
                },
                TTL
            )
            .unwrap()
            .wait(WAIT)
            .unwrap(),
        Err(Error::Runtime(Code::Sequence))
    ));
    let after = until(&host.observer(), |s| s.state.owner.is_none());
    assert_eq!(after.state.revision, state.revision);
    assert_eq!(
        after.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    host.shutdown(WAIT).unwrap();
}
#[test]
fn stale_context_is_a_cached_refusal_and_stopping_one_source_preserves_others() {
    let (mut host, keys) = start();
    let client = connect(&host, 1, false, 60_000);
    let r = client.acquired_state().revision;
    let one = send(&client, 1, r, control(keys[0], Command::Execute(0)));
    let stale = client
        .submit(2, r, control(keys[1], Command::Execute(0)), TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(stale.result, Err(Code::Revision));
    assert_eq!(stale.state.revision, one.revision);
    let two = send(
        &client,
        3,
        stale.state.revision,
        control(keys[3], Command::Execute(0)),
    );
    let stop = send(&client, 4, two.revision, control(keys[0], Command::Stop));
    assert_eq!(stop.sources[0].unwrap().status, Some(Status::Idle));
    assert_eq!(stop.sources[3].unwrap().status, Some(Status::Running));
    host.shutdown(WAIT).unwrap();
}
