use crate::support::*;
use stagemaster_output_port::*;

#[test]
fn missing_quiet_ack_faults_without_claiming_shutdown_and_late_ack_cannot_revive() {
    let (mut port, driver) = setup();
    port.select(source(SourceKind::Local), false, 0).unwrap();
    assert_eq!(port.poll(50), Err(Code::Deadline));
    assert!(!port.state().quiet);
    assert!(port.state().permit.is_none());
    driver.quiet();
    assert_eq!(port.poll(51), Err(Code::Deadline));
    assert!(port.state().quiet);
    assert_eq!(port.state().phase, Phase::Faulted);
    assert_eq!(
        port.with_quiescent(|| panic!("faulted")),
        Err(Code::NotQuiet)
    );
    assert_eq!(
        port.select(source(SourceKind::External), false, 52),
        Err(Code::Deadline)
    );
}
#[test]
fn missing_frame_ack_revokes_output_and_attempts_shutdown() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    port.submit(permit, sample(1, 0, &[4; 512]), 0).unwrap();
    port.poll(0).unwrap();
    assert_eq!(port.poll(50), Err(Code::Deadline));
    assert!(port.state().stopping.is_some());
    assert!(!port.state().quiet);
    assert!(port.state().completed.is_none());
    driver.quiet();
    assert_eq!(port.poll(51), Err(Code::Deadline));
    assert!(port.state().quiet);
    assert_eq!(driver.0.borrow().frames.len(), 1);
}
#[test]
fn backward_clock_invalidates_even_an_otherwise_valid_sample() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    port.submit(permit, sample(1, 10, &[2; 512]), 10).unwrap();
    assert_eq!(
        port.submit(permit, sample(2, 9, &[3; 512]), 9),
        Err(Code::Clock)
    );
    assert!(port.state().pending.is_none());
    assert!(port.state().permit.is_none());
    assert!(!port.state().quiet);
    driver.quiet();
    assert_eq!(port.poll(11), Err(Code::Clock));
    assert!(port.state().quiet);
    assert!(driver.0.borrow().frames.is_empty());
}
#[test]
fn ambiguous_driver_failures_never_count_as_completion_or_quiescence() {
    for failure in 0..3 {
        let (mut port, driver) = setup();
        let permit = ready(&mut port, &driver);
        match failure {
            0 => driver.0.borrow_mut().fail_submit = true,
            1 => driver.0.borrow_mut().fail_stop = true,
            _ => driver.0.borrow_mut().fail_poll = true,
        }
        port.submit(permit, sample(1, 0, &[3; 512]), 0).unwrap();
        if failure == 1 {
            assert_eq!(port.stop(permit, 0), Err(Code::Driver));
        } else {
            assert_eq!(port.poll(0), Err(Code::Driver));
        }
        assert!(port.state().permit.is_none());
        assert!(port.state().pending.is_none());
        assert!(port.state().completed.is_none());
        assert!(!port.state().quiet);
        driver.0.borrow_mut().fail_poll = false;
        driver.quiet();
        assert_eq!(port.poll(1), Err(Code::Driver));
        assert!(port.state().quiet);
        assert!(driver.0.borrow().sent.is_empty());
    }
}
#[test]
fn wrong_completion_kind_faults_instead_of_releasing_the_port() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    port.submit(permit, sample(1, 0, &[3; 512]), 0).unwrap();
    port.poll(0).unwrap();
    let ticket = driver.0.borrow().frames[0].ticket;
    driver.inject(Event::Quiet(ticket));
    assert_eq!(port.poll(1), Err(Code::DriverReport));
    assert!(!port.state().quiet);
    assert!(port.state().completed.is_none());
}
