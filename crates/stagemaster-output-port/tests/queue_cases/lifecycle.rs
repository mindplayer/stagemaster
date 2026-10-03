use crate::{support::*, *};
use stagemaster_output_port::dmx::queued::Fault;

#[test]
fn cancelling_consumer_before_or_after_poll_closes_queue_and_disables_line() {
    for started in [false, true] {
        let mut queue = LocalQueue::new();
        let (mut driver, rx) = queue.split().unwrap();
        let (tx, wire, _) = transmitter();
        driver.submit(tickets()[0], &[1; 512], 100).unwrap();
        let mut run = Box::pin(rx.run(tx));
        if started {
            assert!(poll(run.as_mut()).is_pending());
            assert!(wire.0.borrow().enabled);
        }
        drop(run);
        assert!(!wire.0.borrow().enabled);
        assert_eq!(driver.poll(), Err(Fault::Closed));
        assert_eq!(driver.quiesce(tickets()[1]), Err(Fault::Closed));
        drop(driver);
        assert!(queue.split().is_err());
    }
}

#[test]
fn dropping_producer_terminates_consumer_and_disables_without_fake_receipts() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, _) = transmitter();
    driver.submit(tickets()[0], &[1; 512], 100).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    drop(driver);
    assert!(poll(run.as_mut()).is_ready());
    assert!(!wire.0.borrow().enabled);
}

#[test]
fn idle_freshness_deadline_gates_off_without_autonomous_repeat_or_synthetic_quiet() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let t = tickets();
    driver.submit(t[0], &[1; 512], 100).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert!(wire.0.borrow().enabled);
    time.0.set(100_000);
    assert!(poll(run.as_mut()).is_pending());
    assert!(!wire.0.borrow().enabled);
    assert_eq!(wire.0.borrow().bytes.len(), 513);
    assert_eq!(driver.poll(), Ok(Some(Event::Sent(t[0]))));
    assert_eq!(driver.poll(), Ok(None));
    driver.submit(t[1], &[2; 512], 100).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Ok(Some(Event::Expired(t[1]))));
    assert_eq!(wire.0.borrow().bytes.len(), 513);
}
