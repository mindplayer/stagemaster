#[path = "../../../apps/esp32-player/src/board/watchdog/policy.rs"]
mod policy;
#[path = "../../../apps/esp32-player/src/board/watchdog/progress.rs"]
mod progress;
use policy::{FAILED, Fault, PROGRESS_MS, Policy, STARTUP_MS};
use progress::Progress;

#[test]
fn missing_worker_expires_even_when_supervisor_keeps_running() {
    let mut policy = Policy::new(100);
    for now in (100..100 + STARTUP_MS).step_by(25) {
        assert_eq!(policy.check(now, 0), Ok(()));
    }
    assert_eq!(policy.check(100 + STARTUP_MS, 1), Err(Fault::Deadline));
    assert_eq!(policy.check(100 + STARTUP_MS + 1, 2), Err(Fault::Deadline));
}

#[test]
fn actual_progress_extends_running_budget_but_repeated_observations_do_not() {
    let mut policy = Policy::new(0);
    assert_eq!(policy.check(1, 1), Ok(()));
    for sequence in 2..=400 {
        assert_eq!(policy.check(u64::from(sequence) * 25, sequence), Ok(()));
    }
    assert_eq!(policy.check(10_000 + PROGRESS_MS - 1, 400), Ok(()));
    assert_eq!(
        policy.check(10_000 + PROGRESS_MS, 400),
        Err(Fault::Deadline)
    );
}

#[test]
fn deadline_cannot_be_revived_by_late_new_progress() {
    let mut policy = Policy::new(0);
    policy.check(10, 1).unwrap();
    assert_eq!(policy.check(10 + PROGRESS_MS, 2), Err(Fault::Deadline));
    assert_eq!(policy.check(11 + PROGRESS_MS, 3), Err(Fault::Deadline));
}

#[test]
fn failed_worker_counter_regression_and_sender_exit_latch() {
    for (sequence, expected) in [(FAILED, Fault::Worker), (0, Fault::Sequence)] {
        let mut policy = Policy::new(0);
        policy.check(1, 1).unwrap();
        assert_eq!(policy.check(2, sequence), Err(expected));
        assert_eq!(policy.check(3, 2), Err(expected));
    }
    let mut policy = Policy::new(0);
    assert_eq!(policy.trip(Fault::SenderExited), Err(Fault::SenderExited));
    assert_eq!(policy.check(1, 1), Err(Fault::SenderExited));
}

#[test]
fn clock_rollback_and_overflow_never_feed() {
    let mut policy = Policy::new(100);
    assert_eq!(policy.check(99, 1), Err(Fault::Clock));
    assert_eq!(policy.check(101, 2), Err(Fault::Clock));
    let mut policy = Policy::new(u64::MAX - 10);
    assert_eq!(policy.check(u64::MAX - 10, 1), Err(Fault::Clock));
    // It is possible to approach overflow after a valid initialization.
    let start = u64::MAX - STARTUP_MS;
    let mut policy = Policy::new(start);
    assert_eq!(
        policy.check(u64::MAX - PROGRESS_MS + 1, 1),
        Err(Fault::Clock)
    );
}

#[test]
fn cross_thread_worker_failure_cannot_be_overwritten_by_later_activity() {
    let progress = Progress::new();
    assert_eq!(progress.observed(), 0);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for _ in 0..20_000 {
                progress.beat();
            }
        });
        scope.spawn(|| {
            progress.fail();
            for _ in 0..20_000 {
                progress.beat();
            }
        });
    });
    assert_eq!(progress.observed(), FAILED);
    let mut policy = Policy::new(0);
    assert_eq!(policy.check(0, progress.observed()), Err(Fault::Worker));
}
