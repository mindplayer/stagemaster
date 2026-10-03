use super::support::*;

#[test]
fn initial_and_later_maintenance_require_real_queue_drain_before_storage_access() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    wire.0.borrow_mut().hold = Some("drain");
    output.service(&mut worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    output.service(&mut worker, 0).unwrap();
    assert_eq!(worker.state().mode, Mode::Quiescing);
    assert!(
        worker
            .process(open_command(1), 0, || Some(epoch(1)))
            .result
            .is_err()
    );
    assert_eq!(metrics.mutations.get(), 0);
    wire.0.borrow_mut().hold = None;
    let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
    start(&mut worker, lease, 0);
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    let (info, slots) = render(&mut worker, 0);
    output.publish(&mut worker, info, &slots, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    wire.0.borrow_mut().hold = Some("drain");
    time.0.set(32);
    assert!(poll(run.as_mut()).is_pending());
    apply(&mut worker, lease, Action::Stop, 0).unwrap();
    apply(&mut worker, lease, Action::BeginMaintenance, 0).unwrap();
    output.service(&mut worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    output.service(&mut worker, 0).unwrap();
    assert!(!wire.0.borrow().enabled);
    assert_eq!(worker.state().mode, Mode::Quiescing);
    assert!(
        worker
            .process(open_command(2), 0, || Some(epoch(2)))
            .result
            .is_err()
    );
    assert!(metrics.readers.get() > 0);
    wire.0.borrow_mut().hold = None;
    assert!(poll(run.as_mut()).is_pending());
    output.service(&mut worker, 0).unwrap();
    assert_eq!(worker.state().mode, Mode::Maintenance);
    assert_eq!(metrics.readers.get(), 0);
    open(&mut worker, 2, 0);
}

#[test]
fn cancel_maintenance_does_not_rearm_but_new_start_can_wait_for_old_drain() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
    start(&mut worker, lease, 0);
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    apply(&mut worker, lease, Action::Stop, 0).unwrap();
    apply(&mut worker, lease, Action::BeginMaintenance, 0).unwrap();
    wire.0.borrow_mut().hold = Some("drain");
    output.service(&mut worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    apply(&mut worker, lease, Action::CancelMaintenance, 0).unwrap();
    output.service(&mut worker, 0).unwrap();
    assert_eq!(worker.state().mode, Mode::Operation);
    assert!(output.state().permit.is_none());
    start(&mut worker, lease, 0);
    output.service(&mut worker, 0).unwrap();
    wire.0.borrow_mut().hold = None;
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    assert_eq!(worker.state().mode, Mode::Operation);
    assert!(output.state().permit.is_some());
}

#[test]
fn a_new_start_is_retained_while_expired_source_is_still_quiescing() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
    start(&mut worker, lease, 0);
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    time.0.set(100_000);
    wire.0.borrow_mut().hold = Some("drain");
    output.service(&mut worker, 100).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    start(&mut worker, lease, 100);
    output.service(&mut worker, 100).unwrap();
    output.service(&mut worker, 100).unwrap();
    wire.0.borrow_mut().hold = None;
    settle(&mut output, &mut worker, run.as_mut(), &time, 100);
    settle(&mut output, &mut worker, run.as_mut(), &time, 100);
    assert!(output.state().permit.is_some());
}
