use crate::{support::*, *};
use stagemaster_output_port::dmx::{Clock, Line, Transmitter, queued::QueuedDriver};
use std::cell::RefCell;

struct StopAtCompletion<'a, 'q> {
    wire: Wire,
    driver: &'a RefCell<QueuedDriver<'q, NoopRawMutex>>,
    ticket: Ticket,
    drains: usize,
}
impl Line for StopAtCompletion<'_, '_> {
    type Error = ();
    fn disable(&mut self) {
        self.wire.disable();
    }
    fn enable(&mut self) {
        self.wire.enable();
    }
    async fn break_signal(&mut self, bits: u32) -> Result<(), ()> {
        self.wire.break_signal(bits).await
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        self.wire.write(bytes).await
    }
    async fn drain(&mut self) -> Result<(), ()> {
        self.wire.drain().await?;
        self.drains += 1;
        if self.drains == 2 {
            self.driver.borrow_mut().quiesce(self.ticket).unwrap();
        }
        Ok(())
    }
}
#[test]
fn stop_published_during_final_completion_wins_without_overwriting_new_ticket() {
    let mut queue = LocalQueue::new();
    let (driver, rx) = queue.split().unwrap();
    let driver = RefCell::new(driver);
    let wire = Wire::new();
    let time = Time::default();
    let t = tickets();
    let tx = Transmitter::new(
        StopAtCompletion {
            wire: wire.clone(),
            driver: &driver,
            ticket: t[1],
            drains: 0,
        },
        time.clone(),
        40_000,
    )
    .unwrap();
    driver.borrow_mut().submit(t[0], &[9; 512], 100).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(wire.0.borrow().bytes.len(), 513);
    assert!(!wire.0.borrow().enabled);
    assert_eq!(driver.borrow_mut().poll(), Ok(Some(Event::Quiet(t[1]))));
    assert_eq!(driver.borrow_mut().poll(), Ok(None));
}

struct AdvancingTimer(Time);
impl Clock for AdvancingTimer {
    fn now_us(&self) -> u64 {
        self.0.now_us()
    }
    async fn wait_until_us(&self, until: u64) {
        if until == 100_000 {
            self.0.0.set(until);
        }
        self.0.wait_until_us(until).await;
    }
}
#[test]
fn legitimate_idle_timer_rollover_during_poll_is_not_an_early_timer_fault() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let wire = Wire::new();
    let time = Time::default();
    let tx = Transmitter::new(wire.clone(), AdvancingTimer(time.clone()), 40_000).unwrap();
    let t = tickets()[0];
    driver.submit(t, &[1; 512], 100).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(time.0.get(), 100_000);
    assert_eq!(driver.poll(), Ok(Some(Event::Sent(t))));
    assert!(!wire.0.borrow().enabled);
}
