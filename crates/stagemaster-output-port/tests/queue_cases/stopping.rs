use crate::{support::*, *};
use stagemaster_output_port::dmx::queued::Fault;

#[test]
fn stopping_cancels_every_frame_stage_and_waits_for_actual_drain() {
    for stage in ["break", "mark", "write", "drain"] {
        let mut queue = LocalQueue::new();
        let (mut driver, rx) = queue.split().unwrap();
        let (tx, wire, time) = transmitter();
        let t = tickets();
        if stage == "break" {
            wire.0.borrow_mut().hold = Some("break");
        }
        driver.submit(t[0], &[8; 512], 100).unwrap();
        let mut run = Box::pin(rx.run(tx));
        assert!(poll(run.as_mut()).is_pending());
        if stage == "write" || stage == "drain" {
            wire.0.borrow_mut().hold = Some(stage);
            time.0.set(16);
            assert!(poll(run.as_mut()).is_pending());
        }
        assert!(wire.0.borrow().enabled);
        wire.0.borrow_mut().hold = Some("drain");
        driver.quiesce(t[1]).unwrap();
        assert!(poll(run.as_mut()).is_pending());
        assert!(!wire.0.borrow().enabled, "{stage}");
        assert_eq!(driver.poll(), Ok(None), "no premature Quiet at {stage}");
        wire.0.borrow_mut().hold = None;
        assert!(poll(run.as_mut()).is_pending());
        assert_eq!(driver.poll(), Ok(Some(Event::Quiet(t[1]))));
        assert_eq!(driver.poll(), Ok(None));
    }
}

#[test]
fn latest_stop_replaces_in_flight_stop_without_replaying_older_receipt() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, _) = transmitter();
    let t = tickets();
    wire.0.borrow_mut().hold = Some("drain");
    driver.quiesce(t[0]).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    driver.quiesce(t[1]).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Ok(None));
    wire.0.borrow_mut().hold = None;
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Ok(Some(Event::Quiet(t[1]))));
}

#[test]
fn timeout_is_independent_of_port_polling_and_failure_still_allows_real_quiet() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let t = tickets();
    wire.0.borrow_mut().hold = Some("break");
    driver.submit(t[0], &[8; 512], 100).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(40_000);
    assert!(poll(run.as_mut()).is_pending());
    assert!(!wire.0.borrow().enabled);
    assert_eq!(driver.poll(), Err(Fault::Deadline));
    assert_eq!(driver.submit(t[1], &[9; 512], 100), Err(Fault::Deadline));
    wire.0.borrow_mut().hold = None;
    driver.quiesce(t[2]).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Ok(Some(Event::Quiet(t[2]))));
    assert_eq!(driver.submit(t[1], &[9; 512], 100), Err(Fault::Deadline));
}

#[test]
fn failed_stop_is_not_retried_in_a_busy_loop_or_reported_as_quiet() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, _) = transmitter();
    wire.0.borrow_mut().fail = Some("drain");
    driver.quiesce(tickets()[0]).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Err(Fault::Line));
    for _ in 0..3 {
        assert!(poll(run.as_mut()).is_pending());
    }
    assert_eq!(wire.0.borrow().drain_calls, 1);
    assert_eq!(driver.poll(), Ok(None));
}
