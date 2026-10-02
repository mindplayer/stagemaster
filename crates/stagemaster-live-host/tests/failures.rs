#[path = "support/gate.rs"]
mod gate;
mod support;
use gate::Delayed;
use stagemaster_live::{Change, Command};
use stagemaster_live_host::Patch;
use stagemaster_runtime::{Code, Status};
use stagemaster_runtime_host::{Configuration, Error, Fault, Host, Phase};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use support::*;

#[test]
fn delayed_group_command_expires_without_consuming_serial_or_stopping_other_sources() {
    let (backend, keys) = prepared();
    let (backend, gate) = Delayed::new(backend);
    let mut host = Host::start_backend(
        backend,
        Configuration {
            period: Duration::from_millis(5),
        },
    )
    .unwrap();
    let client = connect(&host, 1, false, 60_000);
    let start = send(
        &client,
        1,
        client.acquired_state().revision,
        control(keys[3], Command::Execute(0)),
    );
    gate.armed.store(true, Ordering::SeqCst);
    gate.entered.recv_timeout(WAIT).unwrap();
    let expired = client
        .submit(
            2,
            start.revision,
            control(keys[3], Command::Stop),
            Duration::from_millis(1),
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(5));
    gate.release.send(()).unwrap();
    assert_eq!(expired.wait(WAIT).unwrap(), Err(Error::Deadline));
    let s = until(&host.observer(), |s| {
        s.state.observed_ms > start.observed_ms
    });
    assert_eq!(s.state.owner.unwrap().serial, 1);
    assert_eq!(s.state.sources[3].unwrap().status, Some(Status::Running));
    send(&client, 2, start.revision, control(keys[3], Command::Stop));
    host.shutdown(WAIT).unwrap();
}
#[test]
fn failed_backend_retracts_observation_through_the_shared_worker() {
    let (backend, keys) = prepared();
    let (backend, gate) = Delayed::new(backend);
    let mut host = Host::start_backend(
        backend,
        Configuration {
            period: Duration::from_millis(5),
        },
    )
    .unwrap();
    let client = connect(&host, 1, false, 60_000);
    send(
        &client,
        1,
        client.acquired_state().revision,
        control(keys[3], Command::Execute(0)),
    );
    let observer = host.observer();
    until(&observer, |s| s.frame.is_some());
    gate.fail.store(true, Ordering::SeqCst);
    let end = Instant::now() + WAIT;
    while observer.phase() != Phase::Faulted {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    let o = observer.read().unwrap();
    assert_eq!(o.fault, Some(Fault::Runtime(Code::Playback)));
    assert!(o.snapshot.is_none());
    assert_eq!(host.shutdown(WAIT).unwrap(), Phase::Faulted);
}
#[test]
fn manual_payload_admission_is_bounded_and_rejects_duplicate_attributes() {
    assert!(
        Patch::new(
            &[Change {
                attribute: 0,
                value: None
            }; 513]
        )
        .is_err()
    );
    assert!(
        Patch::new(
            &[Change {
                attribute: 0,
                value: Some(1)
            }; 2]
        )
        .is_err()
    );
    assert!(
        Patch::new(&[Change {
            attribute: 512,
            value: None
        }])
        .is_err()
    );
    let all: Vec<_> = (0..512)
        .map(|attribute| Change {
            attribute,
            value: Some(0),
        })
        .collect();
    assert!(Patch::new(&all).is_ok());
}
