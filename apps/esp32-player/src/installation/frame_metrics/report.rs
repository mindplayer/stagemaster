use core::fmt;

/// Cumulative per-boot diagnostics, never a physical transmission receipt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub at_us: u64,
    pub attempts: u64,
    pub frames: u64,
    pub running_frames: u64,
    pub offline_frames: u64,
    pub absent: u64,
    pub errors: u64,
    pub clock_errors: u64,
    pub min_gap_us: Option<u64>,
    pub max_gap_us: u64,
    pub over_30ms: u64,
    pub over_50ms: u64,
    pub max_work_us: u64,
    pub backpressure: u64,
    pub commands: u64,
    pub repeated_samples: u64,
    pub instance: u64,
    pub sampled_ms: u64,
    pub elapsed_ms: u64,
    pub running: bool,
    pub online: bool,
    pub fingerprint: u32,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RUNTIME_SAMPLE v=1 at_us={} attempts={} frames={} running_frames={} offline_frames={} absent={} errors={} clock_errors={} min_gap_us={} max_gap_us={} over_30ms={} over_50ms={} max_work_us={} backpressure={} commands={} repeated_samples={} instance={} sampled_ms={} elapsed_ms={} running={} online={} fingerprint={:08x}",
            self.at_us,
            self.attempts,
            self.frames,
            self.running_frames,
            self.offline_frames,
            self.absent,
            self.errors,
            self.clock_errors,
            self.min_gap_us.unwrap_or(0),
            self.max_gap_us,
            self.over_30ms,
            self.over_50ms,
            self.max_work_us,
            self.backpressure,
            self.commands,
            self.repeated_samples,
            self.instance,
            self.sampled_ms,
            self.elapsed_ms,
            u8::from(self.running),
            u8::from(self.online),
            self.fingerprint
        )
    }
}
