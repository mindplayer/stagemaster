use super::support::*;
use crate::{Configuration, Error, Host, Phase, worker};
use stagemaster_package::MAX_LOADER_BYTES;
use stagemaster_runtime::{Action, Request, Runtime};
use std::time::{Duration, Instant};

#[test]
fn empty_and_running_runtimes_cannot_be_transferred_to_a_fresh_host() {
    let empty: Runtime<Reader, _> =
        Runtime::new([2; 16], 0, MAX_LOADER_BYTES, SoftwareAcceptance).unwrap();
    assert!(matches!(
        Host::start(empty, Configuration::default()),
        Err(Error::NotPrepared)
    ));
    let fixture = Fixture::new();
    let mut runtime = fixture.prepared(SoftwareAcceptance);
    let lease = runtime.acquire(grant(3, 60_000), false, 0).unwrap();
    runtime
        .submit(
            Request {
                lease,
                serial: 1,
                expected_revision: runtime.state().revision,
                action: Action::Start {
                    step: runtime.steps()[0].id,
                },
            },
            0,
        )
        .unwrap()
        .result
        .unwrap();
    runtime.release(lease, 0).unwrap();
    assert!(matches!(
        Host::start(runtime, Configuration::default()),
        Err(Error::NotPrepared)
    ));
}

#[test]
fn nonzero_clock_origin_is_preserved_and_clock_exhaustion_faults_closed() {
    let fixture = Fixture::new();
    let mut runtime = fixture.prepared(SoftwareAcceptance);
    runtime.tick(50_000).unwrap();
    let mut host = Host::start(runtime, Configuration::default()).unwrap();
    assert!(until(&host.observer(), |s| s.cycles >= 2).state.observed_ms >= 50_000);
    host.shutdown(WAIT).unwrap();

    let mut runtime = fixture.prepared(SoftwareAcceptance);
    runtime.tick(u64::MAX).unwrap();
    let mut host = Host::start(runtime, Configuration::default()).unwrap();
    let observer = host.observer();
    let deadline = Instant::now() + WAIT;
    while observer.phase() != Phase::Faulted {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(observer.read().unwrap().snapshot.is_none());
    assert_eq!(
        observer.read().unwrap().fault,
        Some(crate::Fault::Runtime(stagemaster_runtime::Code::Exhausted))
    );
    assert_eq!(host.shutdown(WAIT), Ok(Phase::Faulted));
}

#[test]
fn scheduling_keeps_phase_and_skips_missed_slots_without_catchup_bursts() {
    let due = Instant::now();
    let period = Duration::from_millis(25);
    for (elapsed, missed, next) in [
        (0, 0, 25),
        (7, 0, 25),
        (25, 1, 50),
        (87, 3, 100),
        (100_007, 4000, 100_025),
    ] {
        let (actual, count) =
            worker::next_cycle(due, due + Duration::from_millis(elapsed), period).unwrap();
        assert_eq!(count, missed);
        assert_eq!(actual, due + Duration::from_millis(next));
    }
}
