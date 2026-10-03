//! Logic-side driver timings, never a substitute for measuring the output pins.
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_time::Instant;
static MAX_US: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static CANCELLED: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static LATE_US: AtomicU32 = AtomicU32::new(0);
pub(super) struct Operation {
    start: Instant,
    kind: usize,
    completed: bool,
}
impl Operation {
    pub(super) fn begin(kind: usize) -> Self {
        Self {
            start: Instant::now(),
            kind,
            completed: false,
        }
    }
    pub(super) fn complete(mut self) {
        self.completed = true;
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_micros().min(u64::from(u32::MAX)) as u32;
        MAX_US[self.kind].fetch_max(duration, Ordering::Relaxed);
        if !self.completed {
            CANCELLED[self.kind].fetch_add(1, Ordering::Relaxed);
        }
    }
}
pub(super) fn timer(deadline: u64) {
    let late = Instant::now()
        .as_micros()
        .saturating_sub(deadline)
        .min(u64::from(u32::MAX)) as u32;
    LATE_US.fetch_max(late, Ordering::Relaxed);
}
pub fn report() {
    crate::diagnostics::report_line(format_args!(
        "UART TIMING max_break_us={} max_write_us={} max_drain_us={} cancelled={}/{}/{} late_timer_us={}",
        MAX_US[0].load(Ordering::Relaxed),
        MAX_US[1].load(Ordering::Relaxed),
        MAX_US[2].load(Ordering::Relaxed),
        CANCELLED[0].load(Ordering::Relaxed),
        CANCELLED[1].load(Ordering::Relaxed),
        CANCELLED[2].load(Ordering::Relaxed),
        LATE_US.load(Ordering::Relaxed),
    ));
}
