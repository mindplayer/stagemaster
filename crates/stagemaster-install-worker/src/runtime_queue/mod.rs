//! Bounded runtime work, with radio-owned admission and independently ticked execution.
mod endpoint;
mod gateway;
mod outgoing;
pub use endpoint::Endpoint;
pub use gateway::Gateway;

use crate::Epoch;
use stagemaster_device_auth::application::Grant;
use stagemaster_runtime_protocol::{Frame, Offer, Request};

const EXCHANGE_MS: u64 = 5_000;
const WORK_MS: u64 = 30_000;

/// Publish to a synchronized latest-value slot after every gateway operation; clear
/// that slot on any adapter failure, cancellation or Drop. The worker MUST reread
/// the slot, never a cached copy carried by a queued command. Not a wire credential.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live {
    epoch: Epoch,
    grant: Grant,
    admitted_ms: u64,
    until: u64,
}
impl Live {
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }
    /// Evaluate the freshly read slot using the worker's trusted monotonic clock.
    #[must_use]
    pub fn grant(&self, now: u64) -> Option<Grant> {
        (self.admitted_ms <= now && now < self.until).then_some(self.grant)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Work {
    Open(Offer),
    Request(Request),
}
/// Only Gateway constructs commands, after decryption and bounded message validation.
#[derive(Clone, Copy, Debug)]
pub struct Command {
    epoch: Epoch,
    ticket: u64,
    deadline: u64,
    work: Work,
}
impl Command {
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }
}
#[derive(Debug)]
pub struct Completion {
    epoch: Epoch,
    ticket: u64,
    result: Result<Frame, crate::operations::Error>,
}
impl Completion {
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Permission(stagemaster_device_auth::application::Error),
    Protocol(stagemaster_runtime_protocol::Error),
    Worker(crate::operations::Error),
    Order,
    Expired,
    Clock,
    Closed,
}
impl From<stagemaster_device_auth::application::Error> for Error {
    fn from(value: stagemaster_device_auth::application::Error) -> Self {
        Self::Permission(value)
    }
}
impl From<stagemaster_runtime_protocol::Error> for Error {
    fn from(value: stagemaster_runtime_protocol::Error) -> Self {
        Self::Protocol(value)
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Permission(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
            Self::Worker(e) => e.fmt(f),
            Self::Order => f.write_str("设备运行通信顺序异常，请重新连接"),
            Self::Expired => f.write_str("设备运行通信超时，请重新连接并读取状态"),
            Self::Clock => f.write_str("设备运行通信计时异常"),
            Self::Closed => f.write_str("设备运行通信已关闭"),
        }
    }
}
impl core::error::Error for Error {}

fn after(now: u64, duration: u64) -> Result<u64, Error> {
    now.checked_add(duration).ok_or(Error::Clock)
}
