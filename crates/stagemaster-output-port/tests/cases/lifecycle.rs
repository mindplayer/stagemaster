use crate::support::*;
use stagemaster_output_port::*;

#[test]
fn startup_takeover_and_stop_require_the_exact_current_quiet_receipt() {
    let (mut port, driver) = setup();
    assert_eq!(port.state().phase, Phase::Unconfirmed);
    assert_eq!(
        port.with_quiescent(|| panic!("not confirmed")),
        Err(Code::NotQuiet)
    );
    let first = port.select(source(SourceKind::Local), false, 0).unwrap();
    assert!(port.state().permit.is_none());
    driver.quiet();
    port.poll(1).unwrap();
    let local = port.state().permit.unwrap();
    port.submit(local, sample(1, 1, &[4; 512]), 1).unwrap();
    port.poll(1).unwrap();
    port.submit(local, sample(2, 2, &[5; 512]), 2).unwrap();
    assert_eq!(
        port.select(source(SourceKind::External), false, 2),
        Err(Code::Busy)
    );
    let second = port.select(source(SourceKind::External), true, 2).unwrap();
    assert_ne!(first, second);
    assert!(port.state().permit.is_none());
    assert!(port.state().pending.is_none());
    assert_eq!(
        port.submit(local, sample(3, 2, &[6; 512]), 2),
        Err(Code::Permit)
    );
    assert_eq!(
        port.with_quiescent(|| panic!("in flight")),
        Err(Code::NotQuiet)
    );
    driver.inject(Event::Quiet(first));
    driver.finish(3);
    port.poll(3).unwrap();
    assert_eq!(port.state().phase, Phase::Quiescing);
    assert_eq!(driver.0.borrow().frames.len(), 1);
    driver.quiet();
    port.poll(4).unwrap();
    let external = port.state().permit.unwrap();
    assert_ne!(external, local);
    assert_eq!(external.source().kind, SourceKind::External);
    assert_eq!(port.stop(local, 4), Err(Code::Permit));
    port.submit(external, sample(1, 4, &[9; 512]), 4).unwrap();
    port.poll(4).unwrap();
    port.stop(external, 5).unwrap();
    driver.quiet();
    port.poll(6).unwrap();
    assert!(port.state().quiet);
    assert_eq!(port.state().phase, Phase::Idle);
    port.with_quiescent(|| ()).unwrap();
    // The stopped in-flight external frame was cancelled, never completed.
    assert_eq!(port.state().completed.unwrap().permit, local);
    assert_eq!(driver.0.borrow().sent, vec![[4; 512]]);
}
#[test]
fn shutdown_cancels_a_pending_takeover_and_old_proofs_cannot_reopen_maintenance() {
    let (mut port, driver) = setup();
    let ticket = port
        .select(source(SourceKind::Composite), false, 0)
        .unwrap();
    assert_eq!(port.shutdown(1), Ok(Some(ticket)));
    driver.quiet();
    port.poll(2).unwrap();
    port.with_quiescent(|| ()).unwrap();
    assert!(port.state().permit.is_none());
    port.select(source(SourceKind::Local), false, 3).unwrap();
    driver.inject(Event::Quiet(ticket));
    port.poll(3).unwrap();
    assert_eq!(
        port.with_quiescent(|| panic!("old proof")),
        Err(Code::NotQuiet)
    );
    assert!(port.state().permit.is_none());
    driver.quiet();
    port.poll(4).unwrap();
    assert!(port.state().permit.is_some());
    assert_eq!(
        port.with_quiescent(|| panic!("active source")),
        Err(Code::NotQuiet)
    );
}
#[test]
fn source_loss_revokes_output_but_requires_explicit_reacquisition() {
    let (mut port, driver) = setup();
    let old = ready(&mut port, &driver);
    port.submit(old, sample(1, 0, &[7; 512]), 0).unwrap();
    port.poll(0).unwrap();
    driver.finish(1);
    port.poll(1).unwrap();
    port.poll(100).unwrap();
    assert_eq!(port.state().reason, Some(Code::Stale));
    assert_eq!(
        port.submit(old, sample(2, 100, &[8; 512]), 100),
        Err(Code::Permit)
    );
    driver.quiet();
    port.poll(101).unwrap();
    assert_eq!(port.state().phase, Phase::Idle);
    assert_eq!(
        port.submit(old, sample(3, 101, &[9; 512]), 101),
        Err(Code::Permit)
    );
    port.select(source(SourceKind::Local), false, 102).unwrap();
    driver.quiet();
    port.poll(103).unwrap();
    assert_ne!(port.state().permit.unwrap(), old);
    assert_eq!(driver.0.borrow().sent.len(), 1);
}
