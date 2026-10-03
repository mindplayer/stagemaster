use crate::{Clock, Error, Exchange, Instant, Limits, Window};

const MILLION: u128 = 1_000_000;

/// Immutable, one-way measurement. Reads cannot renew it or silently switch clocks.
#[derive(Clone, Copy, Debug)]
pub struct Mapping {
    source: Clock,
    target: Clock,
    reference_ns: u64,
    low: u64,
    high: u64,
    limits: Limits,
}
impl Mapping {
    /// Bound the target's time at source receipt without assuming symmetric network delay.
    /// Positive, bounded oscillator rates and bounded timestamp errors are adapter preconditions.
    /// # Errors
    /// Reject wrong clock epochs, invalid limits, impossible order, excessive delay/uncertainty or overflow.
    pub fn measure(exchange: Exchange, limits: Limits) -> Result<Self, Error> {
        if limits.relative_drift_ppm >= 1_000_000
            || limits.max_round_trip_ns == 0
            || limits.max_age_ns == 0
            || limits.max_uncertainty_ns == 0
        {
            return Err(Error::Limits);
        }
        if exchange.sent.clock != exchange.received.clock
            || exchange.received_at_target.clock != exchange.sent_at_target.clock
        {
            return Err(Error::Clock);
        }
        if exchange.sent.clock.id() == exchange.received_at_target.clock.id() {
            return Err(Error::Identity);
        }
        let elapsed = exchange
            .received
            .nanos
            .checked_sub(exchange.sent.nanos)
            .ok_or(Error::Backwards)?;
        if exchange.sent_at_target.nanos < exchange.received_at_target.nanos {
            return Err(Error::Backwards);
        }
        let error = u128::from(limits.timestamp_error_ns);
        let upper_rate = MILLION + u128::from(limits.relative_drift_ppm);
        let elapsed_bound = u128::from(elapsed) + 2 * error;
        if elapsed_bound > u128::from(limits.max_round_trip_ns) {
            return Err(Error::RoundTrip);
        }
        // Target send may occur after the recorded source receipt by capture error.
        // Include the query's capture error too; all rounding is away from the interval.
        let low = u128::from(exchange.sent_at_target.nanos)
            .checked_sub(error + scale_up(2 * error, upper_rate))
            .ok_or(Error::Overflow)?;
        let high = u128::from(exchange.received_at_target.nanos)
            + error
            + scale_up(elapsed_bound, upper_rate);
        if low > high {
            return Err(Error::Inconsistent);
        }
        let result = Self {
            source: exchange.sent.clock,
            target: exchange.received_at_target.clock,
            reference_ns: exchange.received.nanos,
            low: narrow(low)?,
            high: narrow(high)?,
            limits,
        };
        result.convert(exchange.received)?;
        Ok(result)
    }

    #[must_use]
    pub const fn source(&self) -> Clock {
        self.source
    }
    #[must_use]
    pub const fn target(&self) -> Clock {
        self.target
    }

    /// Convert a current or prospective source instant; later queries widen the drift interval.
    /// # Errors
    /// Reject stale/wrong-domain queries, excessive uncertainty and arithmetic overflow.
    pub fn convert(&self, at: Instant) -> Result<Window, Error> {
        if at.clock != self.source {
            return Err(Error::Clock);
        }
        let delta = at
            .nanos
            .checked_sub(self.reference_ns)
            .ok_or(Error::Backwards)?;
        let age = u128::from(delta) + 2 * u128::from(self.limits.timestamp_error_ns);
        if age >= u128::from(self.limits.max_age_ns) {
            return Err(Error::Expired);
        }
        let drift = u128::from(self.limits.relative_drift_ppm);
        let low = u128::from(self.low) + u128::from(delta) * (MILLION - drift) / MILLION;
        let high = u128::from(self.high) + scale_up(u128::from(delta), MILLION + drift);
        let window = Window {
            earliest: self.target.at(narrow(low)?),
            latest: self.target.at(narrow(high)?),
        };
        if window.uncertainty_ns() > self.limits.max_uncertainty_ns {
            return Err(Error::Uncertain);
        }
        Ok(window)
    }
}
fn scale_up(value: u128, rate: u128) -> u128 {
    (value * rate).div_ceil(MILLION)
}
fn narrow(value: u128) -> Result<u64, Error> {
    u64::try_from(value).map_err(|_| Error::Overflow)
}
