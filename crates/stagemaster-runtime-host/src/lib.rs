//! Independently scheduled, preloaded lighting runtime. No UI, network or physical output.
//!
//! A trusted application prepares and loads a runtime before transferring it here.
//! Its verified operator grant is not inferred from client-provided fields.
//! ```no_run
//! # use stagemaster_runtime_host::{Host, Configuration, Action};
//! # use stagemaster_runtime::{Runtime, PlaybackPolicy, Grant};
//! # use stagemaster_package::ReadAt;
//! # use std::time::Duration;
//! # fn execute<R: ReadAt + Send + 'static, P: PlaybackPolicy + Send + 'static>(
//! # prepared: Runtime<R, P>, verified_grant: Grant) -> Result<(), Box<dyn std::error::Error>> {
//! let step = prepared.steps().first().ok_or("尚未载入执行步骤")?.id;
//! let mut host = Host::start(prepared, Configuration::default())?;
//! let pending = host.connect(verified_grant, false, Duration::from_secs(1))?;
//! let client = pending.wait(Duration::from_secs(1))??;
//! let revision = client.acquired_state().revision;
//! let ticket = client.submit(1, revision, Action::Start { step }, Duration::from_secs(1))?;
//! ticket.wait(Duration::from_secs(1))??.result?;
//! drop(client); // The host still owns execution. This is not Stop.
//! let _observation = host.observer().read()?;
//! host.shutdown(Duration::from_secs(1))?;
//! # Ok(())
//! # }
//! ```
//! The default device queue cannot accept load/install operations:
//! ```compile_fail
//! use stagemaster_runtime_host::Client;
//! fn load_in_playback_queue(client: &Client) {
//!     let _ = client.submit(1, 0, stagemaster_runtime::Action::Load,
//!         std::time::Duration::from_secs(1));
//! }
//! ```
#![forbid(unsafe_code)]
mod backend;
mod client;
mod deadline;
mod host;
mod observation;
mod worker;

pub use backend::{Backend, Device, Profile};
pub use client::{Client, Connection, Ticket, WaitError};
pub use deadline::Deadline;
pub use host::Host;
pub use observation::{Fault, Frame, Observation, Observer, Phase, Snapshot};
use stagemaster_runtime::Code;
use std::time::Duration;

pub const QUEUE_CAPACITY: usize = 32;
pub const COMMANDS_PER_CYCLE: usize = 8;
pub const MAX_COMMAND_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub period: Duration,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            period: Duration::from_millis(25),
        }
    }
}

/// Only already-loaded playback operations can reach the scheduling thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start { step: [u8; 16] },
    Pause,
    Resume,
    Next,
    Stop,
}
impl From<Action> for stagemaster_runtime::Action {
    fn from(action: Action) -> Self {
        match action {
            Action::Start { step } => Self::Start { step },
            Action::Pause => Self::Pause,
            Action::Resume => Self::Resume,
            Action::Next => Self::Next,
            Action::Stop => Self::Stop,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Configuration,
    NotPrepared,
    ThreadSpawn,
    QueueFull,
    InvalidDeadline,
    Deadline,
    Closed,
    ObservationBusy,
    Runtime(Code),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Configuration => "执行刷新周期须为 1～100 毫秒",
            Self::NotPrepared => "执行内容尚未准备为无控制者的待执行状态",
            Self::ThreadSpawn => "无法启动独立执行线程",
            Self::QueueFull => "操作队列已满，本次请求未接纳",
            Self::InvalidDeadline => "操作有效期须大于零且不超过五秒",
            Self::Deadline => "请求已过期，未执行本次操作",
            Self::Closed => "执行宿主已关闭或正在关闭",
            Self::ObservationBusy => "暂时无法取得一致执行快照，请重试",
            Self::Runtime(code) => return code.fmt(f),
        })
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod tests;
