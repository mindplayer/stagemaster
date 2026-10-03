/// A provider's identity and boot/change generation, independent of show position or authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    id: [u8; 16],
    epoch: u64,
}
impl Clock {
    /// # Errors
    /// Reject an absent identity. The adapter owns uniqueness and generation changes.
    pub fn new(id: [u8; 16], epoch: u64) -> Result<Self, Error> {
        if id == [0; 16] {
            return Err(Error::Identity);
        }
        Ok(Self { id, epoch })
    }
    #[must_use]
    pub const fn id(self) -> [u8; 16] {
        self.id
    }
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }
    #[must_use]
    pub const fn at(self, nanos: u64) -> Instant {
        Instant { clock: self, nanos }
    }
}

/// Nanoseconds in exactly one monotonic clock. Intentionally has no cross-domain ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instant {
    pub clock: Clock,
    pub nanos: u64,
}

/// Four timestamps from one authenticated, correlated exchange. Not proof of endpoint trust.
#[derive(Clone, Copy, Debug)]
pub struct Exchange {
    pub sent: Instant,
    pub received_at_target: Instant,
    pub sent_at_target: Instant,
    pub received: Instant,
}

/// Explicit measured/capability bounds, never defaults or estimates inferred from one packet.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_round_trip_ns: u64,
    pub max_age_ns: u64,
    pub max_uncertainty_ns: u64,
    /// Maximum target/source relative rate error, strictly below 1,000,000 ppm.
    pub relative_drift_ppm: u32,
    /// Maximum timestamp capture/quantization error for all four stamps and later queries.
    pub timestamp_error_ns: u64,
}

/// Inclusive possible target instants. Scheduling must consider both ends, not only the midpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub(crate) earliest: Instant,
    pub(crate) latest: Instant,
}
impl Window {
    #[must_use]
    pub const fn earliest(self) -> Instant {
        self.earliest
    }
    #[must_use]
    pub const fn latest(self) -> Instant {
        self.latest
    }
    #[must_use]
    pub const fn midpoint(self) -> Instant {
        self.earliest
            .clock
            .at(self.earliest.nanos + (self.latest.nanos - self.earliest.nanos) / 2)
    }
    #[must_use]
    pub const fn uncertainty_ns(self) -> u64 {
        (self.latest.nanos - self.earliest.nanos).div_ceil(2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Identity,
    Clock,
    Backwards,
    Limits,
    RoundTrip,
    Inconsistent,
    Expired,
    Uncertain,
    Overflow,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Identity => "时钟身份无效或没有独立时间域",
            Self::Clock => "时钟身份或代次不一致",
            Self::Backwards => "时钟观测倒序或早于测量基准",
            Self::Limits => "时钟映射的能力上界无效",
            Self::RoundTrip => "时钟测量往返超过允许范围",
            Self::Inconsistent => "时钟交换顺序与速率上界不一致",
            Self::Expired => "时钟映射已过期，需要重新测量",
            Self::Uncertain => "时钟映射误差超出允许范围",
            Self::Overflow => "时钟换算超出整数范围",
        })
    }
}
