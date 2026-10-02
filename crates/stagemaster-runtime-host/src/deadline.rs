use crate::{Error, MAX_COMMAND_TTL};
use std::time::{Duration, Instant};

/// Process-local admission time. Forward the original Instant across adapters to avoid extending TTL.
#[derive(Clone, Copy, Debug)]
pub enum Deadline {
    After(Duration),
    At(Instant),
}
impl From<Duration> for Deadline {
    fn from(ttl: Duration) -> Self {
        Self::After(ttl)
    }
}
impl From<Instant> for Deadline {
    fn from(time: Instant) -> Self {
        Self::At(time)
    }
}
impl Deadline {
    pub(crate) fn resolve(self) -> Result<Instant, Error> {
        let now = Instant::now();
        match self {
            Self::After(ttl) if !ttl.is_zero() && ttl <= MAX_COMMAND_TTL => {
                now.checked_add(ttl).ok_or(Error::InvalidDeadline)
            }
            Self::After(_) => Err(Error::InvalidDeadline),
            Self::At(time) if time <= now => Err(Error::Deadline),
            Self::At(time) if time.duration_since(now) > MAX_COMMAND_TTL => {
                Err(Error::InvalidDeadline)
            }
            Self::At(time) => Ok(time),
        }
    }
}
