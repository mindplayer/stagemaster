//! Synchronous transition hooks, not an event queue or a second clock.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Advance,
    Execute(usize),
    Next,
    Pause,
    Resume,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activation {
    pub step: usize,
    pub reassert: bool,
    /// Actual delay-end time on the caller's monotonic clock, not the sample time.
    pub at_ms: u64,
}

/// Trusted in-process observer. Callbacks must not block, allocate or perform I/O.
/// The player owns timing; the observer may only seed bounded attribute fade origins.
pub trait Observer {
    /// Timestamped variant; the default preserves existing observers.
    fn activated_at(&mut self, event: Activation, from: &mut [u16]) {
        self.activated(event.step, event.reassert, from);
    }

    /// Start of the last complete omitted cycle. All its activations precede this sample.
    fn cycles_skipped_at(&mut self, _last_cycle_started_ms: u64) {
        self.cycles_skipped();
    }

    /// Called exactly once when a step's delay ends, before rendering its fade.
    /// Explicit execute reasserts; sequential/automatic advance does not.
    fn activated(&mut self, _step: usize, _reassert: bool, _from: &mut [u16]) {}

    /// One or more complete automatic cycles were omitted. Fold metadata once in plan order;
    /// do not replay frames or commands. At this boundary the last step's target is known.
    fn cycles_skipped(&mut self) {}

    /// Explicit stop withdraws any pending ownership and assertions.
    fn stopped(&mut self) {}
}
impl Observer for () {}
