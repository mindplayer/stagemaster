//! Intentionally blocks a real backend only to prove deadline/shutdown/fault containment.
use stagemaster_live_host::{Action, FrameInfo, Live, LiveBackend, State};
use stagemaster_runtime::{Code, Grant, Lease, Receipt, Request};
use stagemaster_runtime_host::Backend;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

pub struct Gate {
    pub armed: Arc<AtomicBool>,
    pub fail: Arc<AtomicBool>,
    pub entered: mpsc::Receiver<()>,
    pub release: mpsc::SyncSender<()>,
}
pub struct Delayed {
    inner: LiveBackend,
    armed: Arc<AtomicBool>,
    fail: Arc<AtomicBool>,
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}
impl Delayed {
    pub fn new(inner: LiveBackend) -> (Self, Gate) {
        let armed = Arc::new(AtomicBool::new(false));
        let fail = Arc::new(AtomicBool::new(false));
        let (signal, entered) = mpsc::sync_channel(1);
        let (release, wait) = mpsc::sync_channel(1);
        (
            Self {
                inner,
                armed: armed.clone(),
                fail: fail.clone(),
                entered: signal,
                release: wait,
            },
            Gate {
                armed,
                fail,
                entered,
                release,
            },
        )
    }
}
impl Backend for Delayed {
    type Profile = Live;
    fn is_prepared(&self) -> bool {
        self.inner.is_prepared()
    }
    fn observed_ms(&self) -> u64 {
        self.inner.observed_ms()
    }
    fn state(&self) -> State {
        self.inner.state()
    }
    fn tick(&mut self, now: u64) -> Result<(), Code> {
        if self.armed.swap(false, Ordering::SeqCst) {
            self.entered.send(()).unwrap();
            self.release.recv_timeout(super::WAIT).unwrap();
        }
        self.inner.tick(now)
    }
    fn render(&self, slots: &mut [u8; 512]) -> Result<Option<FrameInfo>, Code> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(Code::Playback);
        }
        self.inner.render(slots)
    }
    fn acquire(&mut self, g: Grant, t: bool, n: u64) -> Result<Lease, Code> {
        self.inner.acquire(g, t, n)
    }
    fn submit(&mut self, r: Request<Action>, n: u64) -> Result<Receipt<Action, State>, Code> {
        self.inner.submit(r, n)
    }
    fn renew(&mut self, l: Lease, d: u64, n: u64) -> Result<(), Code> {
        self.inner.renew(l, d, n)
    }
    fn release(&mut self, l: Lease, n: u64) -> Result<(), Code> {
        self.inner.release(l, n)
    }
}
