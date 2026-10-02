use super::support::*;
use crate::{Action, Configuration, Error, Host, Phase};
use stagemaster_package::Archive;
use stagemaster_playback::Player;
use stagemaster_runtime::{Code, Denial, Permission, PlaybackPolicy, Status};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[test]
fn clients_disappear_playback_continues_and_reconnection_preserves_the_instance() {
    let fixture = Fixture::new();
    let (mut host, step) = fixture.host();
    let observer = host.observer();
    let client = connect(&host, 3, 100, false);
    let started = start(&client, step);
    assert_eq!(started.result, Ok(()));
    drop(client);
    let later = until(&observer, |s| {
        s.state.owner.is_none() && s.state.elapsed_ms >= 150
    });
    assert_eq!(later.state.instance, started.state.instance);
    assert_eq!(later.state.status, Some(Status::Running));

    // Replay an independent reference at the *actual* sampling timestamp, not a sleep estimate.
    let program = Archive::open(fixture.bytes.as_slice())
        .unwrap()
        .load(fixture.bytes.as_slice(), 0)
        .unwrap();
    let mut reference = Player::new(program.plan, started.state.observed_ms);
    reference.execute(0, started.state.observed_ms).unwrap();
    for _ in 0..12 {
        let sample = until(&observer, |s| s.cycles > later.cycles && s.frame.is_some());
        reference.advance(sample.state.observed_ms).unwrap();
        let mut expected = [0; 512];
        program
            .output
            .render(reference.values(), &mut expected)
            .unwrap();
        assert_eq!(sample.frame.unwrap().slots, expected);
        std::thread::sleep(Duration::from_millis(6));
    }
    let next = connect(&host, 4, 60_000, false);
    let current = until(&observer, |s| {
        s.state.owner.is_some_and(|o| o.principal == [4; 16])
    });
    assert_eq!(current.state.instance, started.state.instance);
    let stopped = next
        .submit(1, current.state.revision, Action::Stop, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(stopped.result, Ok(()));
    assert_eq!(stopped.state.status, Some(Status::Idle));
    assert_eq!(fixture.reads.count.load(Ordering::SeqCst), 0);
    assert_eq!(host.shutdown(WAIT), Ok(Phase::Stopped));
    assert!(observer.read().unwrap().snapshot.is_none());
}

#[test]
fn replaced_controller_cannot_release_renew_or_stop_the_current_controller() {
    let fixture = Fixture::new();
    let (mut host, step) = fixture.host();
    let old = connect(&host, 3, 60_000, false);
    let started = start(&old, step);
    let refused = host
        .connect(grant(4, 60_000), false, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap();
    assert!(matches!(refused, Err(Error::Runtime(Code::Busy))));
    let current = connect(&host, 4, 60_000, true);
    assert_eq!(
        old.release(TTL).unwrap().wait(WAIT).unwrap(),
        Err(Error::Runtime(Code::Lease))
    );
    assert_eq!(
        old.renew(60_000, TTL).unwrap().wait(WAIT).unwrap(),
        Err(Error::Runtime(Code::Lease))
    );
    assert_eq!(
        old.submit(2, started.state.revision, Action::Stop, TTL)
            .unwrap()
            .wait(WAIT)
            .unwrap(),
        Err(Error::Runtime(Code::Lease))
    );
    let state = until(&current.observer(), |s| {
        s.state.owner.is_some_and(|o| o.principal == [4; 16])
    })
    .state;
    assert_eq!(state.instance, started.state.instance);
    current.release(TTL).unwrap().wait(WAIT).unwrap().unwrap();
    assert_eq!(
        until(&host.observer(), |s| s.state.owner.is_none())
            .state
            .status,
        Some(Status::Running)
    );
    host.shutdown(WAIT).unwrap();
}

#[test]
fn repeated_start_returns_the_original_receipt_and_pause_resume_use_the_same_runtime() {
    let fixture = Fixture::new();
    let (mut host, step) = fixture.host();
    let client = connect(&host, 3, 60_000, false);
    let initial = until(&host.observer(), |s| s.state.owner.is_some()).state;
    let action = Action::Start { step };
    let first = client
        .submit(1, initial.revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    let repeated = client
        .submit(1, initial.revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(first, repeated);
    let paused = client
        .submit(2, first.state.revision, Action::Pause, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(paused.result, Ok(()));
    let sample = until(&host.observer(), |s| s.state.status == Some(Status::Paused));
    let after = until(&host.observer(), |s| s.cycles > sample.cycles + 4);
    assert_eq!(sample.state.elapsed_ms, after.state.elapsed_ms);
    let resumed = client
        .submit(3, paused.state.revision, Action::Resume, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    assert_eq!(resumed.result, Ok(()));
    assert_eq!(resumed.state.instance, first.state.instance);
    assert_eq!(fixture.reads.count.load(Ordering::SeqCst), 0);
    host.shutdown(WAIT).unwrap();
}

#[test]
fn a_blocked_observation_slot_skips_publication_instead_of_stalling_execution() {
    let fixture = Fixture::new();
    let (mut host, step) = fixture.host();
    let client = connect(&host, 3, 60_000, false);
    start(&client, step).result.unwrap();
    let observer = host.observer();
    let before = until(&observer, |s| s.state.elapsed_ms > 0);
    let held = observer.shared.snapshot.lock().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(matches!(observer.read(), Err(Error::ObservationBusy)));
    drop(held);
    let after = until(&observer, |s| s.skipped_publications > 0);
    assert!(after.state.elapsed_ms > before.state.elapsed_ms);
    assert!(after.cycles > before.cycles + 1);
    host.shutdown(WAIT).unwrap();
}

struct Reject;
impl PlaybackPolicy for Reject {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        Err(Denial::Expired)
    }
}
struct Fault;
impl PlaybackPolicy for Fault {
    fn authorize(&mut self, _: Permission) -> Result<(), Denial> {
        panic!("注入宿主许可异常")
    }
}

#[test]
fn permission_denial_remains_a_business_receipt_and_does_not_fault_the_host() {
    let fixture = Fixture::new();
    let runtime = fixture.prepared(Reject);
    let step = runtime.steps()[0].id;
    let mut host = Host::start(runtime, Configuration::default()).unwrap();
    let client = connect(&host, 3, 60_000, false);
    assert_eq!(
        start(&client, step).result,
        Err(Code::Permission(Denial::Expired))
    );
    assert_eq!(host.observer().phase(), Phase::Running);
    host.shutdown(WAIT).unwrap();
}

#[test]
fn unexpected_policy_panic_faults_the_host_and_invalidates_frames() {
    let fixture = Fixture::new();
    let runtime = fixture.prepared(Fault);
    let step = runtime.steps()[0].id;
    let mut host = Host::start(runtime, Configuration::default()).unwrap();
    let client = connect(&host, 3, 60_000, false);
    let state = until(&host.observer(), |s| s.state.owner.is_some()).state;
    let ticket = client
        .submit(1, state.revision, Action::Start { step }, TTL)
        .unwrap();
    assert_eq!(ticket.wait(WAIT), Err(crate::WaitError::Unavailable));
    let deadline = Instant::now() + WAIT;
    while host.observer().phase() != Phase::Faulted {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(host.observer().read().unwrap().snapshot.is_none());
    assert!(matches!(client.release(TTL), Err(Error::Closed)));
    assert_eq!(host.shutdown(WAIT), Ok(Phase::Faulted));
    assert_eq!(
        host.observer().read().unwrap().fault,
        Some(crate::Fault::Panic)
    );
}

#[test]
fn invalid_preparation_is_rejected_and_dropping_the_host_revokes_clients() {
    let fixture = Fixture::new();
    let mut runtime = fixture.prepared(SoftwareAcceptance);
    assert!(matches!(
        Host::start(
            fixture.prepared(SoftwareAcceptance),
            Configuration {
                period: Duration::ZERO
            }
        ),
        Err(Error::Configuration)
    ));
    runtime.acquire(grant(3, 60_000), false, 0).unwrap();
    assert!(matches!(
        Host::start(runtime, Configuration::default()),
        Err(Error::NotPrepared)
    ));
    let (host, _) = fixture.host();
    let client = connect(&host, 3, 60_000, false);
    let observer = host.observer();
    drop(host);
    assert!(observer.read().unwrap().snapshot.is_none());
    assert!(matches!(client.release(TTL), Err(Error::Closed)));
    let deadline = Instant::now() + WAIT;
    while observer.phase() != Phase::Stopped {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
}
