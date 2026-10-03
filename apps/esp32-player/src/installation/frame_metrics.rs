//! Fixed-memory statistics for calls to the original worker's tick/render path.
#[path = "frame_metrics/report.rs"]
mod report;
pub use report::Report;
use stagemaster_runtime::{Code, FrameInfo, State, Status};

pub struct Sample {
    pub start_us: u64,
    pub finish_us: u64,
    pub online: bool,
    pub backpressured: bool,
    pub state: State,
}

#[derive(Default)]
pub struct Metrics {
    report: Report,
    previous_start: Option<u64>,
    previous_sample: Option<u64>,
}
impl Metrics {
    pub fn report(&self) -> Report {
        self.report
    }

    pub fn command(&mut self) {
        increment(&mut self.report.commands);
    }

    pub fn failed(&mut self) {
        increment(&mut self.report.errors);
    }

    pub fn record(&mut self, sample: &Sample, frame: &Result<Option<FrameInfo>, Code>, hash: u32) {
        let r = &mut self.report;
        increment(&mut r.attempts);
        r.at_us = sample.finish_us;
        r.online = sample.online;
        r.running = sample.state.status == Some(Status::Running);
        r.instance = sample.state.instance.map_or(0, |v| v.number);
        r.elapsed_ms = sample.state.elapsed_ms;
        r.sampled_ms = sample.state.observed_ms;
        r.fingerprint = 0;
        if sample.backpressured {
            increment(&mut r.backpressure);
        }
        if let Some(previous) = self.previous_start {
            if let Some(gap) = sample.start_us.checked_sub(previous) {
                r.min_gap_us = Some(r.min_gap_us.map_or(gap, |min| min.min(gap)));
                r.max_gap_us = r.max_gap_us.max(gap);
                if gap > 30_000 {
                    increment(&mut r.over_30ms);
                }
                if gap > 50_000 {
                    increment(&mut r.over_50ms);
                }
            } else {
                increment(&mut r.clock_errors);
            }
        }
        self.previous_start = Some(sample.start_us);
        if let Some(work) = sample.finish_us.checked_sub(sample.start_us) {
            r.max_work_us = r.max_work_us.max(work);
        } else {
            increment(&mut r.clock_errors);
        }
        match frame {
            Ok(Some(info)) => {
                increment(&mut r.frames);
                r.sampled_ms = info.sampled_ms;
                r.fingerprint = hash;
                if self.previous_sample == Some(info.sampled_ms) {
                    increment(&mut r.repeated_samples);
                }
                self.previous_sample = Some(info.sampled_ms);
                if r.running && info.instance.is_some() {
                    increment(&mut r.running_frames);
                    if !sample.online {
                        increment(&mut r.offline_frames);
                    }
                }
            }
            Ok(None) => {
                increment(&mut r.absent);
                self.previous_sample = None;
            }
            Err(_) => {
                increment(&mut r.errors);
                self.previous_sample = None;
            }
        }
    }
}

fn increment(value: &mut u64) {
    *value = value.saturating_add(1);
}

/// FNV-1a diagnostic fingerprint of the actual 512 bytes, not cryptographic integrity.
pub fn fingerprint(frame: &[u8; 512]) -> u32 {
    frame.iter().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn counters_saturate_instead_of_wrapping_or_panicking() {
        let mut value = u64::MAX;
        super::increment(&mut value);
        assert_eq!(value, u64::MAX);
    }
}
