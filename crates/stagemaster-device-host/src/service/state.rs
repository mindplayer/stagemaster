use crate::{Phase, Problem, ProblemCode as C, Snapshot};
use std::{
    sync::Mutex,
    time::{Duration, SystemTime},
};
use tokio::{sync::watch, time::Instant};

pub(super) const FRESH_MS: u64 = 4500;

#[derive(Clone, Copy)]
pub(super) struct Stamp {
    monotonic: Instant,
    wall: SystemTime,
}
impl Stamp {
    pub(super) fn now() -> Self {
        Self {
            monotonic: Instant::now(),
            wall: SystemTime::now(),
        }
    }
    pub(super) fn age(self) -> u64 {
        // Wall clock only invalidates connections across suspend/clock changes.
        // It never grants liveness or acts as a playback/licensing clock.
        let wall = self.wall.elapsed().map_or(Duration::MAX, |v| v);
        millis(self.monotonic.elapsed().max(wall))
    }
}
pub(super) fn millis(value: Duration) -> u64 {
    u64::try_from(value.as_millis()).unwrap_or(u64::MAX)
}

pub(super) struct Inner {
    pub(super) snapshot: Snapshot,
    pub(super) cancel: Option<watch::Sender<bool>>,
    pub(super) last_reply: Option<Stamp>,
    pub(super) closed: bool,
    pub(super) installation: Option<(
        crate::InstallationPeer,
        tokio::sync::mpsc::Sender<super::installation::Call>,
    )>,
    pub(super) install_busy: std::sync::Weak<()>,
}
impl Inner {
    pub(super) fn touch(&mut self) {
        self.snapshot.revision += 1;
    }
    pub(super) fn clear_live(&mut self) {
        self.snapshot.diagnostics = None;
        self.snapshot.description = None;
        self.snapshot.round_trip_ms = None;
        self.snapshot.last_reply_age_ms = None;
        self.last_reply = None;
        self.installation = None;
        self.install_busy = std::sync::Weak::new();
    }
    pub(super) fn snapshot(&mut self) -> Snapshot {
        if let Some(last) = self.last_reply {
            let age = last.age();
            if self.snapshot.phase == Phase::Connected && age >= FRESH_MS {
                self.snapshot.phase = Phase::Stopping;
                self.snapshot.problem = Some(Problem::new(C::Timeout));
                self.clear_live();
                if let Some(cancel) = &self.cancel {
                    let _ = cancel.send(true);
                }
                self.touch();
            } else {
                self.snapshot.last_reply_age_ms = Some(age);
            }
        }
        self.snapshot.clone()
    }
}

pub(super) fn update(inner: &Mutex<Inner>, epoch: u32, change: impl FnOnce(&mut Inner)) {
    if let Ok(mut state) = inner.lock()
        && state.snapshot.epoch == epoch
        && state.snapshot.phase != Phase::Stopping
        && !state.closed
    {
        change(&mut state);
        state.touch();
    }
}
