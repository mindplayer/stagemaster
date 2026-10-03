use crate::support::*;
use core::{pin::pin, task::Poll};
use stagemaster_output_port::{Event, Phase, Sample};

#[test]
fn full_packet_is_partial_written_but_not_completed_until_last_stop_bits_drain() {
    let (mut tx, wire, time) = transmitter();
    let ticket = ticket();
    let slots = core::array::from_fn(|i| u8::try_from(i % 256).unwrap());
    let mut send = pin!(tx.send(ticket, &slots, 100));
    assert!(poll(send.as_mut()).is_pending()); // MAB
    assert!(wire.0.borrow().bytes.is_empty());
    time.0.set(15);
    assert!(poll(send.as_mut()).is_pending());
    wire.0.borrow_mut().hold = Some("drain");
    time.0.set(16);
    assert!(poll(send.as_mut()).is_pending());
    let mut packet = vec![0];
    packet.extend_from_slice(&slots);
    assert_eq!(wire.0.borrow().bytes, packet);
    assert_eq!(wire.0.borrow().drain_calls, 2);
    assert!(wire.0.borrow().enabled);
    wire.0.borrow_mut().hold = None;
    assert_eq!(poll(send.as_mut()), Poll::Ready(Ok(Event::Sent(ticket))));
    assert!(wire.0.borrow().enabled); // successful frame leaves marking polarity
    assert_eq!(
        wire.0.borrow().calls[..4],
        ["disable", "drain", "enable", "break"]
    );
}

#[test]
fn port_completion_and_maintenance_use_actual_async_line_results() {
    let (mut tx, wire, time) = transmitter();
    let (mut port, driver) = setup();
    port.select(source(), false, 0).unwrap();
    let stop = driver.0.borrow_mut().stop.take().unwrap();
    let quiet = finish(pin!(tx.quiesce(stop)), &time).unwrap();
    driver.0.borrow_mut().event = Some(quiet);
    port.poll(0).unwrap();
    let permit = port.state().permit.unwrap();
    port.submit(
        permit,
        Sample {
            serial: 1,
            sampled_ms: 0,
            universe: 1,
            slots: &[73; 512],
        },
        0,
    )
    .unwrap();
    port.poll(0).unwrap();
    assert!(port.state().completed.is_none());
    let (t, slots, until) = driver.0.borrow_mut().frame.take().unwrap();
    driver.0.borrow_mut().event = Some(finish(pin!(tx.send(t, &slots, until)), &time).unwrap());
    port.poll(time.0.get() / 1000).unwrap();
    assert_eq!(port.state().completed.unwrap().serial, 1);
    port.stop(permit, time.0.get() / 1000).unwrap();
    assert!(port.with_quiescent(|| panic!("not quiet yet")).is_err());
    let stop = driver.0.borrow_mut().stop.take().unwrap();
    driver.0.borrow_mut().event = Some(finish(pin!(tx.quiesce(stop)), &time).unwrap());
    port.poll(time.0.get() / 1000).unwrap();
    assert_eq!(port.state().phase, Phase::Idle);
    assert!(port.with_quiescent(|| {}).is_ok());
    assert!(!wire.0.borrow().enabled);
}

#[test]
fn expiry_is_rechecked_after_old_uart_work_finishes() {
    let (mut tx, wire, time) = transmitter();
    let ticket = ticket();
    wire.0.borrow_mut().hold = Some("drain");
    let mut send = pin!(tx.send(ticket, &[2; 512], 2));
    assert!(poll(send.as_mut()).is_pending());
    time.0.set(2000);
    wire.0.borrow_mut().hold = None;
    assert_eq!(poll(send.as_mut()), Poll::Ready(Ok(Event::Expired(ticket))));
    assert!(!wire.0.borrow().enabled);
    assert!(wire.0.borrow().bytes.is_empty());
    assert!(!wire.0.borrow().calls.iter().any(|s| s == "break"));
}
