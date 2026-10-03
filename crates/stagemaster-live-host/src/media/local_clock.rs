use stagemaster_runtime::Code;
use stagemaster_time::{Clock, Exchange, Mapping};
use std::time::Instant;

/// Correlate original consumer timestamps produced in this same process using `std::time::Instant`.
/// Exact here means one shared software clock, NOT acoustic presentation or independent device accuracy.
#[derive(Clone, Copy, Debug)]
pub struct LocalClock {
    anchor: stagemaster_runtime_host::Clock,
    provider: Clock,
    target: Clock,
    validity_ns: u64,
}
impl LocalClock {
    /// # Errors
    /// Refuse identical clock identities or zero validity. Caller binds these to the prepared group/host.
    pub fn new(
        anchor: stagemaster_runtime_host::Clock,
        provider: Clock,
        target: Clock,
        validity_ns: u64,
    ) -> Result<Self, Code> {
        if provider.id() == target.id() || validity_ns == 0 {
            return Err(Code::Clock);
        }
        Ok(Self {
            anchor,
            provider,
            target,
            validity_ns,
        })
    }
    /// Convert the ORIGINAL local sample instant. Repeated calls with an old instant stay old.
    /// This adapter must never be used for timestamps from an independent hardware/network clock.
    /// # Errors
    /// Refuse pre-host timestamps or overflow. Group admission still checks actual sample age.
    pub fn map(self, at: Instant) -> Result<(stagemaster_time::Instant, Mapping), Code> {
        let nanos = self.anchor.at_ns(at)?;
        let sample = self.provider.at(nanos);
        let target = self.target.at(nanos);
        let mapping = Mapping::measure(
            Exchange {
                sent: sample,
                received: sample,
                received_at_target: target,
                sent_at_target: target,
            },
            stagemaster_time::Limits {
                max_round_trip_ns: 1,
                max_age_ns: self.validity_ns,
                max_uncertainty_ns: 1,
                relative_drift_ppm: 0,
                timestamp_error_ns: 0,
            },
        )
        .map_err(|_| Code::Clock)?;
        Ok((sample, mapping))
    }
}
