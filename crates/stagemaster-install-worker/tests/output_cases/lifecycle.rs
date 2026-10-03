use super::support::*;

#[test]
fn load_does_not_transmit_until_start_then_pause_stop_and_disconnection_keep_core_frames() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
    let (info, slots) = render(&mut worker, 0);
    assert!(!output.publish(&mut worker, info, &slots, 0).unwrap());
    assert!(wire.0.borrow().bytes.is_empty());
    assert!(output.state().quiet);
    start(&mut worker, lease, 0);
    settle(&mut output, &mut worker, run.as_mut(), &time, 0);
    worker.release(lease, 0).unwrap();
    let mut frames = Vec::new();
    for n in 0..20 {
        let now = n * 25;
        time.0.set(now * 1000 + 16);
        let (info, slots) = render(&mut worker, now);
        assert!(output.publish(&mut worker, info, &slots, now).unwrap());
        assert!(poll(run.as_mut()).is_pending());
        time.0.set(now * 1000 + 32);
        assert!(poll(run.as_mut()).is_pending());
        output.service(&mut worker, now).unwrap();
        let data = &wire.0.borrow().bytes;
        assert_eq!(data.len(), usize::try_from(n + 1).unwrap() * 513);
        assert_eq!(data[data.len() - 513], 0);
        assert_eq!(&data[data.len() - 512..], slots);
        assert_eq!(output.state().completed.unwrap().serial, n + 1);
        frames.push(slots);
    }
    assert!(frames.windows(2).any(|f| f[0] != f[1]));
    let lease = acquire(&mut worker, 500);
    apply(&mut worker, lease, Action::Pause, 500).unwrap();
    let (_, paused) = render(&mut worker, 500);
    for now in [500, 525] {
        time.0.set(now * 1000);
        let (info, slots) = render(&mut worker, now);
        assert_eq!(slots, paused);
        assert!(output.publish(&mut worker, info, &slots, now).unwrap());
        settle(&mut output, &mut worker, run.as_mut(), &time, now);
    }
    apply(&mut worker, lease, Action::Stop, 550).unwrap();
    time.0.set(550_000);
    let (info, defaults) = render(&mut worker, 550);
    assert!(info.instance.is_none());
    assert!(output.publish(&mut worker, info, &defaults, 550).unwrap());
    settle(&mut output, &mut worker, run.as_mut(), &time, 550);
    let data = &wire.0.borrow().bytes;
    assert_eq!(&data[data.len() - 512..], defaults);
}

#[test]
fn disabling_or_expiring_output_does_not_rearm_the_same_instance() {
    for expire in [false, true] {
        let (_dir, mut worker, _metrics, bytes) = fixture();
        let mut queue = Queue::<NoopRawMutex>::new();
        let (driver, rx) = queue.split().unwrap();
        let (tx, _wire, time) = transmitter();
        let mut run = Box::pin(rx.run(tx));
        let mut output = LocalOutput::new(config(), driver, 0).unwrap();
        let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
        start(&mut worker, lease, 0);
        settle(&mut output, &mut worker, run.as_mut(), &time, 0);
        if !expire {
            output.disable(0).unwrap();
        }
        settle(&mut output, &mut worker, run.as_mut(), &time, 100);
        let (info, slots) = render(&mut worker, 100);
        assert!(!output.publish(&mut worker, info, &slots, 100).unwrap());
        assert!(output.state().quiet);
        start(&mut worker, lease, 100);
        settle(&mut output, &mut worker, run.as_mut(), &time, 100);
        assert!(output.state().permit.is_some());
    }
}

#[test]
fn stop_before_start_has_acquired_output_cancels_pending_activation() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    let mut queue = Queue::<NoopRawMutex>::new();
    let (driver, rx) = queue.split().unwrap();
    let (tx, wire, _time) = transmitter();
    let mut run = Box::pin(rx.run(tx));
    let mut output = LocalOutput::new(config(), driver, 0).unwrap();
    let lease = prepare(&mut worker, &bytes, &mut output, run.as_mut());
    start(&mut worker, lease, 0);
    output.service(&mut worker, 0).unwrap();
    apply(&mut worker, lease, Action::Stop, 0).unwrap();
    // Quiet was produced before the host observed Stop; it must not grant a
    // now-cancelled initial activation when service consumes the receipt.
    assert!(poll(run.as_mut()).is_pending());
    output.service(&mut worker, 0).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    output.service(&mut worker, 0).unwrap();
    assert!(output.state().permit.is_none());
    assert!(wire.0.borrow().bytes.is_empty());
}
