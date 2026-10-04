#[path = "support/gate.rs"]
mod gate;
mod support;
use stagemaster_live::{BatchCommand, Change, Command, Key};
use stagemaster_live_host::{Action, Batch, LiveBackend, Patch};
use stagemaster_runtime::{Code, Lease, Request, Status};
use stagemaster_runtime_host::{Backend, Configuration, Error, Host};
use std::{sync::atomic::Ordering, time::Duration};
use support::*;

fn batch(keys: &[Key], command: BatchCommand) -> Action {
    Action::Batch {
        batch: Batch::new(keys).unwrap(),
        command,
    }
}

#[test]
fn batch_has_one_revision_and_receipt_replay_does_not_repeat_or_release_manual() {
    let (mut backend, keys) = prepared();
    let lease = backend.acquire(grant(1, 60_000), false, 0).unwrap();
    let mut serial = 0;
    let mut apply = |backend: &mut stagemaster_live_host::LiveBackend, action, now| {
        serial += 1;
        backend
            .submit(
                Request {
                    lease,
                    serial,
                    expected_revision: backend.state().revision,
                    action,
                },
                now,
            )
            .unwrap()
    };
    for key in [keys[0], keys[3]] {
        apply(&mut backend, control(key, Command::Execute(0)), 0)
            .result
            .unwrap();
    }
    apply(
        &mut backend,
        Action::Patch {
            source: keys[2],
            patch: Patch::new(&[Change {
                attribute: 2,
                value: Some(0),
            }])
            .unwrap(),
        },
        0,
    )
    .result
    .unwrap();
    let revision = backend.state().revision;
    let request = Request {
        lease,
        serial: 4,
        expected_revision: revision,
        action: batch(&[keys[0], keys[3]], BatchCommand::Pause),
    };
    let paused = backend.submit(request.clone(), 25).unwrap();
    paused.result.unwrap();
    assert_eq!(paused.state.revision, revision + 1);
    for index in [0, 3] {
        assert_eq!(
            paused.state.sources[index].unwrap().status,
            Some(Status::Paused)
        );
    }
    backend.tick(100).unwrap();
    assert_eq!(backend.submit(request, 100).unwrap(), paused);
    let resumed = backend
        .submit(
            Request {
                lease,
                serial: 5,
                expected_revision: paused.state.revision,
                action: batch(&[keys[0], keys[3]], BatchCommand::Resume),
            },
            100,
        )
        .unwrap();
    resumed.result.unwrap();
    let stopped = backend
        .submit(
            Request {
                lease,
                serial: 6,
                expected_revision: resumed.state.revision,
                action: batch(&[keys[0]], BatchCommand::Stop),
            },
            110,
        )
        .unwrap();
    stopped.result.unwrap();
    assert_eq!(stopped.state.sources[0].unwrap().status, Some(Status::Idle));
    assert_eq!(
        stopped.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    assert_eq!(stopped.state.manual_values[2].as_ref().unwrap()[2], Some(0));
    assert_manual_target_rejected(&mut backend, &keys, lease, stopped.state.revision);
}

fn assert_manual_target_rejected(
    backend: &mut LiveBackend,
    keys: &[Key],
    lease: Lease,
    revision: u64,
) {
    let rejected = backend
        .submit(
            Request {
                lease,
                serial: 7,
                expected_revision: revision,
                action: batch(&[keys[3], keys[2]], BatchCommand::Stop),
            },
            111,
        )
        .unwrap();
    assert_eq!(rejected.result, Err(Code::State));
    assert_eq!(
        rejected.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    assert_eq!(
        rejected.state.manual_values[2].as_ref().unwrap()[2],
        Some(0)
    );
}

#[test]
fn stale_revision_lost_lease_and_payload_budget_do_not_control_any_target() {
    let (mut backend, keys) = prepared();
    assert_eq!(Batch::new(&[]), Err(Code::Budget));
    assert_eq!(Batch::new(&[keys[0]; 65]), Err(Code::Budget));
    assert_eq!(Batch::new(&[keys[0]; 2]), Err(Code::Selection));
    let lease = backend.acquire(grant(1, 60_000), false, 0).unwrap();
    let current = backend
        .submit(
            Request {
                lease,
                serial: 1,
                expected_revision: backend.state().revision,
                action: control(keys[3], Command::Execute(0)),
            },
            0,
        )
        .unwrap()
        .state;
    let action = Action::Batch {
        batch: Batch::new(&[keys[0], keys[3]]).unwrap(),
        command: BatchCommand::Stop,
    };
    let stale = backend
        .submit(
            Request {
                lease,
                serial: 2,
                expected_revision: 0,
                action: action.clone(),
            },
            10,
        )
        .unwrap();
    assert_eq!(stale.result, Err(Code::Revision));
    assert_eq!(stale.state.revision, current.revision);
    assert_eq!(
        stale.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    let next = backend.acquire(grant(2, 5), true, 10).unwrap();
    assert_eq!(
        backend.submit(
            Request {
                lease,
                serial: 3,
                expected_revision: backend.state().revision,
                action: action.clone()
            },
            11
        ),
        Err(Code::Lease)
    );
    assert_eq!(
        backend.submit(
            Request {
                lease: next,
                serial: 1,
                expected_revision: backend.state().revision,
                action
            },
            16
        ),
        Err(Code::Lease)
    );
    assert_eq!(
        backend.state().sources[3].unwrap().status,
        Some(Status::Running)
    );
}

#[test]
fn queued_expired_batch_never_partially_stops_or_consumes_the_serial() {
    let (backend, keys) = prepared();
    let (backend, gate) = gate::Delayed::new(backend);
    assert!(!gate.fail.load(Ordering::SeqCst));
    let mut host = Host::start_backend(
        backend,
        Configuration {
            period: Duration::from_millis(5),
        },
    )
    .unwrap();
    let client = connect(&host, 1, false, 60_000);
    let running = send(
        &client,
        1,
        client.acquired_state().revision,
        control(keys[3], Command::Execute(0)),
    );
    gate.armed.store(true, Ordering::SeqCst);
    gate.entered.recv_timeout(WAIT).unwrap();
    let action = Action::Batch {
        batch: Batch::new(&[keys[0], keys[3]]).unwrap(),
        command: BatchCommand::Stop,
    };
    let pending = client
        .submit(
            2,
            running.revision,
            action.clone(),
            Duration::from_millis(1),
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(5));
    gate.release.send(()).unwrap();
    assert_eq!(pending.wait(WAIT).unwrap(), Err(Error::Deadline));
    let observed = until(&host.observer(), |s| {
        s.state.observed_ms > running.observed_ms
    });
    assert_eq!(observed.state.owner.unwrap().serial, 1);
    assert_eq!(
        observed.state.sources[3].unwrap().status,
        Some(Status::Running)
    );
    let stopped = send(&client, 2, running.revision, action);
    assert_eq!(stopped.sources[3].unwrap().status, Some(Status::Idle));
    host.shutdown(WAIT).unwrap();
}
