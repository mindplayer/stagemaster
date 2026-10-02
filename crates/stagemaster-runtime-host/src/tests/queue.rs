use super::support::*;
use crate::{
    Action, Configuration, Error, Host, Phase, QUEUE_CAPACITY, WaitError,
    client::{Command, Envelope, Ingress},
    observation::Shared,
    worker,
};
use stagemaster_runtime::{Code, Permission, PlaybackPolicy};
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[test]
fn admission_and_receipts_are_bounded_without_waiting_for_consumers() {
    let fixture = Fixture::new();
    let mut runtime = fixture.prepared(SoftwareAcceptance);
    let shared = Arc::new(Shared::new(runtime.state()));
    let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
    let ingress = Ingress { sender, shared };
    assert!(matches!(
        ingress.send(Duration::ZERO, |reply| Command::Acquire {
            grant: grant(3, 60_000),
            takeover: false,
            reply
        }),
        Err(Error::InvalidDeadline)
    ));
    let mut tickets = Vec::new();
    for _ in 0..QUEUE_CAPACITY {
        tickets.push(
            ingress
                .send(TTL, |reply| Command::Acquire {
                    grant: grant(3, 60_000),
                    takeover: false,
                    reply,
                })
                .unwrap(),
        );
    }
    assert!(matches!(
        ingress.send(TTL, |reply| Command::Acquire {
            grant: grant(3, 60_000),
            takeover: false,
            reply
        }),
        Err(Error::QueueFull)
    ));
    assert_eq!(tickets[0].wait(Duration::ZERO), Err(WaitError::Timeout));
    worker::dispatch(&mut runtime, receiver.try_recv().unwrap(), 0);
    let acquired = tickets[0].wait(WAIT).unwrap().unwrap();
    assert_eq!(runtime.state().owner.unwrap().lease, acquired.lease);
    // Dropped reply receivers cannot hold up dispatch, even when acquisition is refused.
    drop(tickets);
    for envelope in receiver.try_iter() {
        worker::dispatch(&mut runtime, envelope, 0);
    }
    assert_eq!(runtime.state().owner.unwrap().lease, acquired.lease);
}

#[test]
fn expired_commands_do_not_consume_the_core_serial_and_can_be_reissued() {
    let fixture = Fixture::new();
    let mut runtime = fixture.prepared(SoftwareAcceptance);
    let lease = runtime.acquire(grant(3, 60_000), false, 0).unwrap();
    let request = stagemaster_runtime::Request {
        lease,
        serial: 1,
        expected_revision: runtime.state().revision,
        action: Action::Start {
            step: runtime.steps()[0].id,
        },
    };
    let (reply, received) = mpsc::sync_channel(1);
    worker::dispatch(
        &mut runtime,
        Envelope {
            deadline: Instant::now()
                .checked_sub(Duration::from_millis(1))
                .unwrap(),
            command: Command::Submit { request, reply },
        },
        1,
    );
    assert_eq!(received.recv().unwrap(), Err(Error::Deadline));
    assert_eq!(runtime.state().owner.unwrap().serial, 0);
    assert!(runtime.state().instance.is_none());
    let (reply, received) = mpsc::sync_channel(1);
    worker::dispatch(
        &mut runtime,
        Envelope {
            deadline: Instant::now() + TTL,
            command: Command::Submit { request, reply },
        },
        2,
    );
    assert_eq!(received.recv().unwrap().unwrap().result, Ok(()));
    assert_eq!(runtime.state().owner.unwrap().serial, 1);
}

struct Gate {
    entered: mpsc::SyncSender<()>,
    leave: mpsc::Receiver<()>,
}
impl PlaybackPolicy for Gate {
    fn authorize(&mut self, _: Permission) -> Result<(), stagemaster_runtime::Denial> {
        self.entered.send(()).unwrap();
        self.leave.recv_timeout(WAIT).unwrap();
        Ok(())
    }
}

#[test]
fn shutdown_bypasses_a_full_queue_and_timeout_keeps_the_same_worker_handle() {
    let fixture = Fixture::new();
    let (entered, signal) = mpsc::sync_channel(1);
    let (release, leave) = mpsc::sync_channel(1);
    // Deliberately violate the bounded policy contract to test timeout/fault containment.
    let runtime = fixture.prepared(Gate { entered, leave });
    let step = runtime.steps()[0].id;
    let mut host = Host::start(runtime, Configuration::default()).unwrap();
    let client = connect(&host, 3, 60_000, false);
    let state = until(&host.observer(), |s| s.state.owner.is_some()).state;
    let start = client
        .submit(1, state.revision, Action::Start { step }, TTL)
        .unwrap();
    signal.recv_timeout(WAIT).unwrap();
    for _ in 0..QUEUE_CAPACITY {
        drop(client.renew(60_000, TTL).unwrap());
    }
    assert!(matches!(client.release(TTL), Err(Error::QueueFull)));
    assert_eq!(host.shutdown(Duration::ZERO), Err(WaitError::Timeout));
    assert_eq!(host.observer().phase(), Phase::Stopping);
    assert!(host.observer().read().unwrap().snapshot.is_none());
    assert!(matches!(client.release(TTL), Err(Error::Closed)));
    release.send(()).unwrap();
    assert_eq!(host.shutdown(WAIT), Ok(Phase::Stopped));
    assert_eq!(host.shutdown(WAIT), Ok(Phase::Stopped));
    assert_eq!(start.wait(WAIT).unwrap().unwrap().result, Ok(()));
    // A request already executing may finish; queue shutdown is not physical rollback.
}

#[test]
fn invalid_renewal_does_not_change_the_owner() {
    let fixture = Fixture::new();
    let (mut host, _) = fixture.host();
    let client = connect(&host, 3, 60_000, false);
    assert_eq!(
        client.renew(0, TTL).unwrap().wait(WAIT).unwrap(),
        Err(Error::Runtime(Code::Identity))
    );
    assert!(
        until(&host.observer(), |s| s.state.owner.is_some())
            .state
            .owner
            .is_some()
    );
    host.shutdown(WAIT).unwrap();
}

#[test]
fn acquisition_receipt_supports_commands_even_when_observation_is_blocked() {
    let fixture = Fixture::new();
    let (mut host, step) = fixture.host();
    let observer = host.observer();
    let held = observer.shared.snapshot.lock().unwrap();
    let client = host
        .connect(grant(3, 60_000), false, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert!(client.acquired_state().owner.is_some());
    assert!(matches!(observer.read(), Err(Error::ObservationBusy)));
    let receipt = start(&client, step);
    assert_eq!(receipt.result, Ok(()));
    assert!(receipt.state.instance.is_some());
    drop(held);
    host.shutdown(WAIT).unwrap();
}
