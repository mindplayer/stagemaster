use stagemaster_runtime::Code;
use std::time::Instant;

/// Local process anchor shared by the actual scheduling thread and trusted media adapters.
/// It is not a hardware clock, network timestamp or measurement of presentation latency.
#[derive(Clone, Copy, Debug)]
pub struct Clock {
    origin: Instant,
    base_ms: u64,
}
impl Clock {
    pub(crate) fn new(base_ms: u64) -> Self {
        Self {
            origin: Instant::now(),
            base_ms,
        }
    }
    /// Preserve the original sample instant. Do not pass a later read time for cached media.
    /// # Errors
    /// Reject instants preceding this host and unrepresentable nanosecond values.
    pub fn at_ns(self, at: Instant) -> Result<u64, Code> {
        let elapsed = at.checked_duration_since(self.origin).ok_or(Code::Clock)?;
        self.base_ms
            .checked_mul(1_000_000)
            .and_then(|base| base.checked_add(u64::try_from(elapsed.as_nanos()).ok()?))
            .ok_or(Code::Exhausted)
    }
    /// # Errors
    /// Reject instants preceding this host or exhausted millisecond range.
    pub fn at_ms(self, at: Instant) -> Result<u64, Code> {
        let elapsed = at.checked_duration_since(self.origin).ok_or(Code::Clock)?;
        self.base_ms
            .checked_add(u64::try_from(elapsed.as_millis()).map_err(|_| Code::Exhausted)?)
            .ok_or(Code::Exhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn sample_conversion_preserves_origin_units_and_overflow_errors() {
        let clock = Clock::new(9);
        let sample = clock.origin + Duration::from_micros(1234);
        assert_eq!(clock.at_ns(sample), Ok(10_234_000));
        assert_eq!(clock.at_ms(sample), Ok(10));
        assert_eq!(clock.at_ns(sample), clock.at_ns(sample));
        assert_eq!(
            clock.at_ns(clock.origin.checked_sub(Duration::from_nanos(1)).unwrap()),
            Err(Code::Clock)
        );
        let exhausted = Clock {
            base_ms: u64::MAX,
            ..clock
        };
        assert_eq!(exhausted.at_ns(clock.origin), Err(Code::Exhausted));
        assert_eq!(exhausted.at_ms(sample), Err(Code::Exhausted));
    }
}
