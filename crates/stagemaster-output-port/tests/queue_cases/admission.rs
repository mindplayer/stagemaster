use crate::{support::*, *};
use stagemaster_output_port::dmx::queued::Fault;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Wake, Waker},
};

#[derive(Default)]
struct Counter(AtomicUsize);
impl Wake for Counter {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn queue_copies_full_frame_and_backpressures_until_completion_is_consumed() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let t = tickets();
    let mut run = Box::pin(rx.run(tx));
    let counter = Arc::new(Counter::default());
    let waker = Waker::from(counter.clone());
    assert!(
        run.as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
    );
    let mut slots = [41; 512];
    driver.submit(t[0], &slots, 100).unwrap();
    slots.fill(99);
    assert_eq!(
        counter.0.load(Ordering::SeqCst),
        1,
        "admission wakes the independent task"
    );
    assert_eq!(driver.submit(t[1], &slots, 100), Err(Fault::Busy));
    assert_eq!(driver.poll(), Ok(None));
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.submit(t[1], &slots, 100), Err(Fault::Busy));
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(wire.0.borrow().bytes, [vec![0], vec![41; 512]].concat());
    assert_eq!(driver.submit(t[1], &slots, 100), Err(Fault::Busy));
    assert_eq!(driver.poll(), Ok(Some(Event::Sent(t[0]))));
    driver.submit(t[1], &slots, 100).unwrap();
}

#[test]
fn priority_stop_discards_unsent_frame_and_unread_completion() {
    let mut queue = LocalQueue::new();
    let (mut driver, rx) = queue.split().unwrap();
    let (tx, wire, time) = transmitter();
    let t = tickets();
    driver.submit(t[0], &[1; 512], 100).unwrap();
    driver.quiesce(t[1]).unwrap();
    let mut run = Box::pin(rx.run(tx));
    assert!(poll(run.as_mut()).is_pending());
    assert!(wire.0.borrow().bytes.is_empty());
    assert_eq!(driver.poll(), Ok(Some(Event::Quiet(t[1]))));
    driver.submit(t[0], &[2; 512], 100).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    time.0.set(16);
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(wire.0.borrow().bytes.len(), 513);
    driver.quiesce(t[2]).unwrap();
    assert!(poll(run.as_mut()).is_pending());
    assert_eq!(driver.poll(), Ok(Some(Event::Quiet(t[2]))));
    assert_eq!(driver.poll(), Ok(None));
    assert!(!wire.0.borrow().enabled);
}
