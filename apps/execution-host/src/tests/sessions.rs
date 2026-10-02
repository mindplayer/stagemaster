use super::*;
use crate::wire::{Command, Decimal};
use serde_json::json;

fn input(serial: u64) -> Input {
    Input {
        serial: Decimal(serial),
        ttl_ms: 1000,
        command: Command::Release {},
    }
}
fn completed() -> Completion {
    Completion {
        outcome: json!({"kind":"released"}),
        change: Change::Clear,
    }
}
fn code<T>(result: Result<T, Failure>) -> &'static str {
    result.err().unwrap().1
}

#[test]
fn contiguous_serials_retain_one_receipt_and_never_replay_old_work() {
    let mut r = Registry::new();
    let now = Instant::now();
    let id = r.create(now).unwrap();
    assert_eq!(code(r.begin(id, input(2), now, true)), "sequence");
    assert!(matches!(
        r.begin(id, input(1), now, true).unwrap(),
        Admission::New(_)
    ));
    assert!(matches!(
        r.begin(id, input(1), now, false).unwrap(),
        Admission::Existing(_)
    ));
    let mut different = input(1);
    different.ttl_ms = 500;
    assert_eq!(code(r.begin(id, different, now, true)), "duplicateConflict");
    assert_eq!(code(r.begin(id, input(2), now, true)), "busy");
    r.finish(id, 1, completed(), now);
    assert_eq!(r.receipt(id, 1, now).unwrap().status, "complete");
    assert!(matches!(
        r.begin(id, input(2), now, true).unwrap(),
        Admission::New(_)
    ));
    assert_eq!(code(r.begin(id, input(1), now, true)), "notRetained");
    assert_eq!(code(r.receipt(id, 1, now)), "notRetained");
}
#[test]
fn session_capacity_expiry_and_pending_work_are_bounded() {
    let mut r = Registry::new();
    let now = Instant::now();
    let id = r.create(now).unwrap();
    r.begin(id, input(1), now, true).unwrap();
    for _ in 1..MAX_SESSIONS {
        r.create(now).unwrap();
    }
    assert_eq!(code(r.create(now)), "sessionLimit");
    let later = now + IDLE;
    assert!(r.receipt(id, 1, later).is_ok());
    for _ in 1..MAX_SESSIONS {
        r.create(later).unwrap();
    }
    assert_eq!(r.entries.len(), MAX_SESSIONS);
    r.finish(id, 1, completed(), later);
    assert_eq!(code(r.receipt(id, 1, later + IDLE)), "sessionMissing");
    assert_eq!(
        code(r.begin(id, input(1), later + IDLE, true)),
        "sessionMissing"
    );
}
#[test]
fn maximum_serial_and_shutdown_do_not_reset_a_session() {
    let mut r = Registry::new();
    let now = Instant::now();
    let id = r.create(now).unwrap();
    r.begin(id, input(1), now, true).unwrap();
    r.finish(id, 1, completed(), now);
    let record = r.entries.get_mut(&id).unwrap().record.as_mut().unwrap();
    record.input.serial = Decimal(u64::MAX);
    record.view.serial = Decimal(u64::MAX);
    assert_eq!(code(r.begin(id, input(1), now, true)), "notRetained");
    r.accepting = false;
    assert_eq!(code(r.create(now)), "closed");
    assert_eq!(code(r.begin(id, input(u64::MAX), now, true)), "closed");
}
