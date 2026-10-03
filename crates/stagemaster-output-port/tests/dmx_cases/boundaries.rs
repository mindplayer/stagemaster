use crate::support::*;
use core::{future::poll_fn, pin::pin, task::Poll};
use stagemaster_output_port::{
    Event,
    dmx::{Clock, Error, Transmitter},
};

#[test]
fn final_uart_failure_is_not_sent_and_dropping_a_successful_owner_gates_off() {
    let (mut tx, wire, time) = transmitter();
    let t = ticket();
    {
        let mut send = pin!(tx.send(t, &[1; 512], 100));
        assert!(poll(send.as_mut()).is_pending());
        time.0.set(16);
        wire.0.borrow_mut().fail = Some("drain");
        assert_eq!(poll(send.as_mut()), Poll::Ready(Err(Error::Line(()))));
    }
    assert!(!wire.0.borrow().enabled);
    wire.0.borrow_mut().fail = None;
    assert_eq!(
        finish(pin!(tx.send(t, &[2; 512], 100)), &time),
        Ok(Event::Sent(t))
    );
    assert!(wire.0.borrow().enabled);
    drop(tx);
    assert!(!wire.0.borrow().enabled);
}

#[test]
fn invalid_budget_expiry_at_entry_and_time_overflow_do_not_enable_line() {
    let wire = Wire::new();
    let time = Time::default();
    wire.0.borrow_mut().enabled = true;
    assert!(matches!(
        Transmitter::new(wire.clone(), time.clone(), 1),
        Err(Error::Budget)
    ));
    assert!(!wire.0.borrow().enabled);
    let mut tx = Transmitter::new(wire.clone(), time.clone(), 40000).unwrap();
    let t = ticket();
    assert_eq!(
        finish(pin!(tx.send(t, &[1; 512], 0)), &time),
        Ok(Event::Expired(t))
    );
    time.0.set(u64::MAX - 100);
    assert_eq!(
        finish(pin!(tx.send(t, &[1; 512], u64::MAX)), &time),
        Err(Error::Exhausted)
    );
    assert!(!wire.0.borrow().enabled);
    assert!(wire.0.borrow().bytes.is_empty());
}

struct EarlyMark(Time);
impl Clock for EarlyMark {
    fn now_us(&self) -> u64 {
        self.0.now_us()
    }
    async fn wait_until_us(&self, until: u64) {
        poll_fn(|_| {
            if until < 1000 || self.now_us() >= until {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}
#[test]
fn an_early_mark_timer_cannot_shorten_mab() {
    let wire = Wire::new();
    let clock = EarlyMark(Time::default());
    let mut tx = Transmitter::new(wire.clone(), clock, 40000).unwrap();
    assert_eq!(
        poll(pin!(tx.send(ticket(), &[1; 512], 100))),
        Poll::Ready(Err(Error::Clock))
    );
    assert!(wire.0.borrow().bytes.is_empty());
    assert!(!wire.0.borrow().enabled);
}
