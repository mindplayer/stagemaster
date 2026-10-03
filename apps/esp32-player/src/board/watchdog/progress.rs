//! Single worker owns publication. Saturation fails; old activity never wraps alive.
use super::policy::FAILED;
use core::sync::atomic::{AtomicU32, Ordering};

pub struct Progress(AtomicU32);
impl Progress {
    pub const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn beat(&self) {
        let _ = self
            .0
            .fetch_update(Ordering::Release, Ordering::Relaxed, |n| {
                (n != FAILED).then(|| n.saturating_add(1))
            });
    }
    pub fn fail(&self) {
        self.0.store(FAILED, Ordering::Release);
    }
    pub fn observed(&self) -> u32 {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustion_latches_failure_instead_of_wrapping_to_startup() {
        let progress = Progress(AtomicU32::new(FAILED - 1));
        progress.beat();
        assert_eq!(progress.observed(), FAILED);
        progress.beat();
        assert_eq!(progress.observed(), FAILED);
    }
}
