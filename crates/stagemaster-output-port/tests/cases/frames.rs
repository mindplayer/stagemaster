use crate::support::*;
use stagemaster_output_port::*;

#[test]
fn pending_is_one_complete_latest_frame_and_acceptance_is_not_completion() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    port.submit(permit, sample(1, 0, &[1; 512]), 0).unwrap();
    assert!(port.state().submitted.is_none());
    port.poll(0).unwrap();
    assert!(port.state().completed.is_none());
    for i in 2..=20_u8 {
        port.submit(permit, sample(u64::from(i), 1, &[i; 512]), 1)
            .unwrap();
        port.poll(1).unwrap();
    }
    assert_eq!(driver.0.borrow().frames.len(), 1);
    assert_eq!(port.state().accepted.unwrap().serial, 20);
    driver.finish(2);
    port.poll(2).unwrap();
    assert_eq!(port.state().completed.unwrap().serial, 1);
    assert_eq!(port.state().submitted.unwrap().serial, 20);
    assert!(port.state().pending.is_none());
    driver.finish(3);
    port.poll(3).unwrap();
    assert_eq!(driver.0.borrow().sent, vec![[1; 512], [20; 512]]);
}
#[test]
fn invalid_frames_never_replace_pending_or_extend_freshness() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    port.submit(permit, sample(10, 10, &[1; 512]), 10).unwrap();
    for (value, expected) in [
        (
            Sample {
                universe: 2,
                ..sample(11, 11, &[2; 512])
            },
            Code::Universe,
        ),
        (sample(0, 11, &[2; 512]), Code::Sequence),
        (sample(10, 11, &[2; 512]), Code::Sequence),
        (sample(11, 21, &[2; 512]), Code::Future),
        (sample(11, 9, &[2; 512]), Code::Stale),
    ] {
        assert_eq!(port.submit(permit, value, 20), Err(expected));
        assert_eq!(port.state().accepted.unwrap().serial, 10);
    }
    port.poll(20).unwrap();
    driver.finish(21);
    port.poll(21).unwrap();
    port.poll(110).unwrap();
    assert!(port.state().permit.is_none());
    assert_eq!(driver.0.borrow().sent, vec![[1; 512]]);
}
#[test]
fn driver_drops_a_frame_that_expires_before_it_can_start() {
    let (mut port, driver) = setup();
    let permit = ready(&mut port, &driver);
    // Older sample is still within its budget when admitted.
    port.submit(permit, sample(1, 0, &[8; 512]), 80).unwrap();
    port.poll(80).unwrap();
    driver.finish(100);
    port.poll(100).unwrap();
    assert!(driver.0.borrow().sent.is_empty());
    assert!(port.state().completed.is_none());
    assert!(port.state().permit.is_none());
    driver.quiet();
    port.poll(101).unwrap();
    assert!(port.state().quiet);
}
#[test]
fn late_frame_receipts_do_not_complete_a_new_owners_frame() {
    let (mut port, driver) = setup();
    let local = ready(&mut port, &driver);
    port.submit(local, sample(1, 0, &[3; 512]), 0).unwrap();
    port.poll(0).unwrap();
    let old = driver.0.borrow().frames[0].ticket;
    port.select(source(SourceKind::External), true, 1).unwrap();
    driver.quiet();
    port.poll(2).unwrap();
    let new = port.state().permit.unwrap();
    port.submit(new, sample(1, 2, &[4; 512]), 2).unwrap();
    port.poll(2).unwrap();
    driver.inject(Event::Sent(old));
    port.poll(3).unwrap();
    assert!(port.state().completed.is_none());
    assert_eq!(port.state().in_flight.unwrap().permit, new);
    driver.finish(4);
    port.poll(4).unwrap();
    assert_eq!(port.state().completed.unwrap().permit, new);
}
