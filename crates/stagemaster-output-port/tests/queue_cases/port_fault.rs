use crate::{support::*, *};
use stagemaster_output_port::dmx::queued::Fault;

#[test]
fn original_port_latches_line_fault_but_observes_late_quiet_without_reopening_maintenance() {
    let mut queue = LocalQueue::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut port = Port::new(
        Config {
            boot: [1; 16],
            port: 1,
            universe: 1,
            max_age_ms: 100,
            ack_timeout_ms: 50,
        },
        driver,
        0,
    )
    .unwrap();
    port.select(source(), false, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    port.poll(0).unwrap();
    let permit = port.state().permit.unwrap();
    port.submit(
        permit,
        Sample {
            serial: 1,
            sampled_ms: 0,
            universe: 1,
            slots: &[8; 512],
        },
        0,
    )
    .unwrap();
    port.poll(0).unwrap();
    wire.0.borrow_mut().fail = Some("write");
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert!(!wire.0.borrow().enabled);
    assert_eq!(port.poll(0), Err(Code::Driver));
    assert!(port.state().permit.is_none());
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(port.poll(0), Err(Code::Driver));
    assert!(port.state().quiet);
    assert_eq!(port.state().phase, Phase::Faulted);
    assert!(port.state().completed.is_none());
    assert_eq!(
        port.with_quiescent(|| panic!("must remain closed")),
        Err(Code::NotQuiet)
    );
}

#[test]
fn fixed_queue_and_future_fit_declared_software_budget_and_reject_clock_overflow_before_output() {
    let mut queue = LocalQueue::new();
    let bytes = core::mem::size_of_val(&queue);
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, _) = transmitter();
    let run = rx.run(tx);
    let future_bytes = core::mem::size_of_val(&run);
    eprintln!("Queue bytes={bytes}, host receiver future bytes={future_bytes}");
    assert!(bytes <= 1024);
    assert!(future_bytes <= 8192);
    assert_eq!(
        driver.submit(tickets()[0], &[1; 512], u64::MAX),
        Err(Fault::Exhausted)
    );
    assert!(wire.0.borrow().bytes.is_empty());
    assert!(!wire.0.borrow().enabled);
    drop(run);
}
