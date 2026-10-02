use core::fmt;

/// The host must choose a fresh boot identity for every construction of a port.
/// Durations are explicit host policy, not protocol constants or commercial limits.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub boot: [u8; 16],
    pub port: u16,
    pub universe: u16,
    pub max_age_ms: u64,
    pub ack_timeout_ms: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Local,
    External,
    Composite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: [u8; 16],
    pub kind: SourceKind,
}
/// Opaque operation identity: drivers echo it, hosts cannot construct it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub(crate) boot: [u8; 16],
    pub(crate) port: u16,
    pub(crate) counter: u64,
}
/// Output authority, not a user control lease or commercial permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permit {
    pub(crate) ticket: Ticket,
    pub(crate) source: Source,
}
impl Permit {
    #[must_use]
    pub const fn source(self) -> Source {
        self.source
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Sample<'a> {
    pub serial: u64,
    pub sampled_ms: u64,
    pub universe: u16,
    pub slots: &'a [u8; 512],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameId {
    pub permit: Permit,
    pub serial: u64,
    pub sampled_ms: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Unconfirmed,
    Quiescing,
    Ready,
    Idle,
    Faulted,
}
/// Historical frame metadata is not evidence that a physical lamp is lit or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub phase: Phase,
    pub permit: Option<Permit>,
    pub quiet: bool,
    pub stopping: Option<Ticket>,
    pub pending: Option<FrameId>,
    pub in_flight: Option<FrameId>,
    pub accepted: Option<FrameId>,
    pub submitted: Option<FrameId>,
    pub completed: Option<FrameId>,
    pub reason: Option<Code>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Identity,
    Clock,
    Busy,
    Permit,
    Sequence,
    Universe,
    Future,
    Stale,
    Deadline,
    Driver,
    DriverReport,
    Exhausted,
    NotQuiet,
}
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identity => "输出端口身份、来源或时间预算无效",
            Self::Clock => "输出时钟倒退，已撤销输出许可",
            Self::Busy => "输出端口正在使用或等待停止确认",
            Self::Permit => "输出许可已失效，请读取当前端口状态",
            Self::Sequence => "输出帧号必须严格递增且不能为零",
            Self::Universe => "输出帧线路与端口不符",
            Self::Future => "输出采样时间晚于当前时间",
            Self::Stale => "输出来源未及时更新，需重新选择来源",
            Self::Deadline => "输出驱动未按期完成，无法确认端口状态",
            Self::Driver => "输出驱动故障，已撤销输出许可",
            Self::DriverReport => "输出驱动回执与当前操作不符",
            Self::Exhausted => "输出计数或时钟范围耗尽，不能回绕",
            Self::NotQuiet => "端口尚未确认静默，不能开始维护",
        })
    }
}
impl core::error::Error for Code {}
