//! Process-local software consumption, deliberately not a serializable DAC timestamp.
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

static NEXT_SOURCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Consumption {
    /// Nonzero source instance, unique only within this process lifetime.
    pub instance: u64,
    /// Complete consumed sample frames, independent of loops or material position.
    pub frames: u64,
    pub sample_rate: u32,
    /// Original consumer-side monotonic publication time, never the later read time.
    pub at: Instant,
}

pub(crate) fn next_instance() -> Result<u64, String> {
    allocate(&NEXT_SOURCE)
}
fn allocate(counter: &AtomicU64) -> Result<u64, String> {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
        .map_err(|_| "音频消费实例编号已耗尽，请重新启动应用".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_instance_counter_never_reuses_an_old_identity() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(allocate(&counter).unwrap(), u64::MAX - 1);
        assert!(allocate(&counter).is_err());
        assert!(allocate(&counter).is_err());
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
