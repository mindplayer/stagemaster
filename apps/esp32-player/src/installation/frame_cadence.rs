//! Scheduling state, separate from diagnostic counters and independent of command meaning.
#[derive(Default)]
pub struct Cadence {
    sampled_us: Option<u64>,
}
impl Cadence {
    /// Refresh before potentially slow work unless a recent sample already exists.
    /// Clock rollback still reaches the real Runtime, which rejects it.
    pub fn before_command(&self, now_us: u64) -> bool {
        self.sampled_us
            .is_none_or(|last| now_us.checked_sub(last).is_none_or(|age| age >= 10_000))
    }
    pub fn sampled(&mut self, now_us: u64) {
        self.sampled_us = Some(now_us);
    }
}
