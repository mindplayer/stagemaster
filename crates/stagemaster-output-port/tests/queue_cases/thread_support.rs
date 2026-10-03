use core::{
    future::poll_fn,
    pin::pin,
    task::{Context, Poll, Waker},
};
use stagemaster_output_port::dmx::{Clock, Line};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::Wake,
    thread,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Timers {
    now: AtomicU64,
    waits: Mutex<Vec<(u64, Waker)>>,
}
#[derive(Clone, Default)]
pub struct Time(Arc<Timers>);
impl Time {
    pub fn advance(&self, now: u64) {
        let mut waits = self.0.waits.lock().unwrap();
        self.0.now.store(now, Ordering::SeqCst);
        let mut ready = Vec::new();
        waits.retain(|(until, wake)| {
            if *until <= now {
                ready.push(wake.clone());
                false
            } else {
                true
            }
        });
        drop(waits);
        for wake in ready {
            wake.wake();
        }
    }
}
impl Clock for Time {
    fn now_us(&self) -> u64 {
        self.0.now.load(Ordering::SeqCst)
    }
    async fn wait_until_us(&self, until: u64) {
        poll_fn(|cx| {
            let mut waits = self.0.waits.lock().unwrap();
            if self.now_us() >= until {
                return Poll::Ready(());
            }
            if !waits
                .iter()
                .any(|(t, w)| *t == until && w.will_wake(cx.waker()))
            {
                waits.push((until, cx.waker().clone()));
            }
            Poll::Pending
        })
        .await;
    }
}

#[derive(Default)]
pub struct Trace {
    pub enabled: bool,
    pub bytes: Vec<u8>,
    pub hold: bool,
    pub waiting: Option<Waker>,
}
#[derive(Clone, Default)]
pub struct Wire(pub Arc<Mutex<Trace>>);
impl Line for Wire {
    type Error = ();
    fn disable(&mut self) {
        self.0.lock().unwrap().enabled = false;
    }
    fn enable(&mut self) {
        self.0.lock().unwrap().enabled = true;
    }
    async fn break_signal(&mut self, bits: u32) -> Result<(), ()> {
        assert_eq!(bits, 30);
        Ok(())
    }
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        let count = bytes.len().min(77);
        self.0
            .lock()
            .unwrap()
            .bytes
            .extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    async fn drain(&mut self) -> Result<(), ()> {
        poll_fn(|cx| {
            let mut state = self.0.lock().unwrap();
            if state.hold {
                state.waiting = Some(cx.waker().clone());
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
        .await
    }
}

struct ThreadWake {
    thread: thread::Thread,
    notified: AtomicBool,
}
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.notified.store(true, Ordering::SeqCst);
        self.thread.unpark();
    }
}
pub fn run(future: impl Future<Output = ()>) {
    let signal = Arc::new(ThreadWake {
        thread: thread::current(),
        notified: AtomicBool::new(true),
    });
    let waker = Waker::from(signal.clone());
    let mut future = pin!(future);
    let limit = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(
            Instant::now() < limit,
            "lost wake or consumer failed to shut down"
        );
        if signal.notified.swap(false, Ordering::SeqCst)
            && future
                .as_mut()
                .poll(&mut Context::from_waker(&waker))
                .is_ready()
        {
            return;
        }
        // Only actual wakers cause a repoll. Timeout/spurious park returns do not.
        thread::park_timeout(Duration::from_millis(10));
    }
}
pub fn wait(mut ready: impl FnMut() -> bool) {
    let limit = Instant::now() + Duration::from_secs(2);
    while !ready() {
        assert!(
            Instant::now() < limit,
            "independent worker did not progress"
        );
        thread::sleep(Duration::from_micros(50));
    }
}
