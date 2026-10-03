use super::support::*;

#[test]
fn failed_drain_never_grants_worker_maintenance() {
    let (_dir, mut worker, metrics, _bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, _) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    wire.0.borrow_mut().fail = Some("drain");
    output.service(&mut worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    assert!(output.service(&mut worker, 0).is_err());
    assert_eq!(worker.state().mode, Mode::Quiescing);
    assert!(
        worker
            .process(open_command(1), 0, || Some(epoch(1)))
            .result
            .is_err()
    );
    assert_eq!(metrics.mutations.get(), 0);
    assert!(output.fault().is_some());
}

#[test]
fn old_or_foreign_snapshot_withdraws_output_and_fault_cannot_be_cleared_by_late_quiet() {
    for mismatch in ["boot", "revision", "program", "instance", "time"] {
        let (_dir, mut worker, _metrics, bytes) = fixture();
        let mut queue = Queue::<NoopRawMutex>::new();
        let (driver, rx) = queue.split().unwrap();
        let (tx, wire, time) = transmitter();
        let mut run = Box::pin(rx.run(tx));
        let mut output = LocalOutput::new(config(), driver, 0).unwrap();
        let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
        start(&mut worker, lease, 0);
        settle(&mut output, &mut worker, run.as_mut(), &time, 0);
        let (mut info, slots) = render(&mut worker, 0);
        match mismatch {
            "boot" => info.boot = [99; 16],
            "revision" => info.revision += 1,
            "program" => info.program.id = [99; 16],
            "instance" => info.instance = None,
            "time" => info.sampled_ms += 1,
            _ => unreachable!(),
        }
        assert_eq!(
            output.publish(&mut worker, info, &slots, 0),
            Err(Error::Snapshot)
        );
        assert!(poll(run.as_mut()).is_pending());
        assert_eq!(output.service(&mut worker, 0), Err(Error::Snapshot));
        assert!(output.state().quiet);
        assert!(wire.0.borrow().bytes.is_empty());
        apply(&mut worker, lease, Action::Stop, 0).unwrap();
        apply(&mut worker, lease, Action::BeginMaintenance, 0).unwrap();
        assert_eq!(output.service(&mut worker, 0), Err(Error::Snapshot));
        assert_eq!(worker.state().mode, Mode::Quiescing);
    }
}
