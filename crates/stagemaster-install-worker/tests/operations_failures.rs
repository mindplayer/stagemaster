#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
use maintenance_support::{fixture, install};
use operation_support::{Peer, access, all};
use stagemaster_install_worker::operations::{Connection, Error, Failure, Operation};
use stagemaster_runtime::{Action, Code, Status};

#[test]
fn wrong_session_stale_revision_and_conflicting_retry_do_not_reexecute() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut peer = Peer::new(&device, 9, all(), 0);
    peer.prepare(&mut device);
    let step = device.steps()[0].id;
    let started = peer.apply(&mut device, Action::Start { step }, 0);
    let owner = device.state().owner;
    let mut wrong = started.request;
    wrong.session = [99; 16];
    assert_eq!(peer.request(&mut device, wrong, 0), Err(Error::Identity));
    assert_eq!(device.state().owner, owner);
    wrong = started.request;
    wrong.operation = Operation::Apply(Action::Stop);
    assert_eq!(peer.request(&mut device, wrong, 0), Err(Error::Sequence));
    assert_eq!(device.state().owner, None);
    assert_eq!(device.state().instance, started.state.instance);
    assert_eq!(device.state().status, Some(Status::Running));
    let mut new = Peer::new(&device, 9, all(), 0);
    let acquired = new
        .send(
            &mut device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: false,
            },
            0,
        )
        .unwrap();
    let mut stale = acquired.request;
    stale.id = 2;
    stale.operation = Operation::Apply(Action::Stop);
    let rejected = new.request(&mut device, stale, 0).unwrap();
    assert_eq!(rejected.result, Err(Failure::Runtime(Code::Revision)));
    assert_eq!(device.state().instance, started.state.instance);
    assert_eq!(new.request(&mut device, stale, 0).unwrap(), rejected);
}

#[test]
fn permissions_and_clock_are_rechecked_before_and_after_potentially_slow_loading() {
    for lose_access in [false, true] {
        let (_dir, mut device, _metrics, bytes) = fixture();
        install(&mut device, &bytes);
        let mut peer = Peer::new(&device, 9, all(), 0);
        peer.prepare(&mut device);
        // Enter/rebind maintenance so the next load really reads from the installed source.
        peer.apply(&mut device, Action::BeginMaintenance, 0);
        device
            .confirm_quiescent(device.quiescence_request().unwrap(), 0)
            .unwrap();
        peer.send(&mut device, Operation::FinishMaintenance, 0)
            .unwrap()
            .result
            .unwrap();
        let entry = &device.catalog()[0];
        let key = stagemaster_runtime::ProgramKey {
            kind: entry.kind,
            id: entry.id,
        };
        let selected = peer.apply(&mut device, Action::Select(key), 0);
        let mut request = selected.request;
        request.id += 1;
        request.expected_revision = device.state().revision;
        request.operation = Operation::Apply(Action::Load);
        let mut checks = 0;
        let mut clocks = 0;
        let result = peer.connection.process(
            &mut device,
            request,
            || {
                clocks += 1;
                if clocks == 1 || lose_access {
                    1
                } else {
                    10_000
                }
            },
            |now| {
                checks += 1;
                if lose_access && checks == 2 {
                    peer.access.revoke();
                }
                peer.access.grant(now).ok()
            },
        );
        assert_eq!(result, Err(Error::Obsolete));
        assert_eq!(device.state().loaded, Some(key)); // Completed work is not a rollback.
        assert_eq!(device.state().instance, None);
        assert_eq!(device.state().owner, None);
        assert_eq!(
            peer.connection.poll(&mut device, 10_000, |_| None),
            Err(Error::Closed)
        );
    }
}

#[test]
fn failed_load_retries_need_a_new_request_after_the_read_source_recovers() {
    let (_dir, mut device, metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut peer = Peer::new(&device, 9, all(), 0);
    peer.send(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    )
    .unwrap()
    .result
    .unwrap();
    peer.send(&mut device, Operation::FinishMaintenance, 0)
        .unwrap()
        .result
        .unwrap();
    let entry = &device.catalog()[0];
    let key = stagemaster_runtime::ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    peer.apply(&mut device, Action::Select(key), 0);
    metrics.fail_read.set(true);
    let failed = peer
        .send(&mut device, Operation::Apply(Action::Load), 0)
        .unwrap();
    assert_eq!(failed.result, Err(Failure::Runtime(Code::Read)));
    assert_eq!(device.state().loaded, None);
    metrics.fail_read.set(false);
    assert_eq!(
        peer.request(&mut device, failed.request, 0).unwrap(),
        failed
    );
    assert_eq!(device.state().loaded, None);
    peer.apply(&mut device, Action::Load, 0);
    assert_eq!(device.state().loaded, Some(key));
}

#[test]
fn expired_or_revoked_connections_release_only_input_and_cannot_be_revived() {
    for revoked in [false, true] {
        let (_dir, mut device, _metrics, bytes) = fixture();
        install(&mut device, &bytes);
        let mut peer = Peer::new(&device, 9, all(), 0);
        peer.prepare(&mut device);
        let step = device.steps()[0].id;
        let started = peer.apply(&mut device, Action::Start { step }, 0);
        if revoked {
            peer.access.revoke();
        }
        let now = if revoked { 100 } else { 6000 };
        assert_eq!(
            peer.connection
                .poll(&mut device, now, |t| peer.access.grant(t).ok()),
            Err(Error::Obsolete)
        );
        device.tick(now).unwrap();
        assert_eq!(device.state().instance, started.state.instance);
        assert_eq!(device.state().owner, None);
        assert!(device.render(&mut [0; 512]).unwrap().is_some());
        let mut access = access(device.state().boot, 9, all(), now);
        assert_eq!(
            peer.connection
                .poll(&mut device, now, |t| access.grant(t).ok()),
            Err(Error::Closed)
        );
    }
}

#[test]
fn foreign_boot_and_backward_time_do_not_admit_or_restart_a_runtime() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut wrong = access([99; 16], 9, all(), 0);
    assert!(matches!(
        Connection::open(&device, 0, |t| wrong.grant(t).ok()),
        Err(Error::Identity)
    ));
    let mut peer = Peer::new(&device, 9, all(), 0);
    peer.prepare(&mut device);
    device.tick(100).unwrap();
    assert_eq!(
        peer.connection
            .poll(&mut device, 99, |t| peer.access.grant(t).ok()),
        Err(Error::Clock)
    );
    assert_eq!(device.state().observed_ms, 100);
    assert_eq!(device.state().owner, None);
}

#[test]
fn requested_input_lease_cannot_exceed_the_remaining_operation_permission() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut peer = Peer::new(&device, 9, all(), 0);
    let reply = peer
        .send(
            &mut device,
            Operation::Acquire {
                duration_ms: 10_001,
                takeover: false,
            },
            0,
        )
        .unwrap();
    assert_eq!(reply.result, Err(Failure::Runtime(Code::Identity)));
    assert_eq!(device.state().owner, None);
    peer.send(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    )
    .unwrap()
    .result
    .unwrap();
    let owner = device.state().owner;
    let reply = peer
        .send(
            &mut device,
            Operation::Renew {
                duration_ms: 10_001,
            },
            0,
        )
        .unwrap();
    assert_eq!(reply.result, Err(Failure::Runtime(Code::Identity)));
    assert_eq!(device.state().owner, owner);
}
