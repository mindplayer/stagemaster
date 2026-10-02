//! A single device playback authority. No transport, storage driver or physical output.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod authority;
mod runtime;
use core::fmt;
pub use runtime::{MaintenanceError, Runtime};
use stagemaster_install::{Commit, Identity};
use stagemaster_package::{Id, Kind};
pub use stagemaster_playback::Status;

pub const MAX_LEASE_MS: u64 = 60_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgramKey {
    pub kind: Kind,
    pub id: Id,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lease {
    pub boot: Id,
    pub epoch: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Panel,
    Remote,
}
/// Trusted host input, never evidence of authentication merely because it is well-formed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    pub principal: Id,
    pub origin: Origin,
    pub duration_ms: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub lease: Lease,
    pub principal: Id,
    pub origin: Origin,
    pub expires_ms: u64,
    pub serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instance {
    pub boot: Id,
    pub number: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quiescence {
    boot: Id,
    revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Maintenance {
    boot: Id,
    revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Operation,
    Quiescing,
    Maintenance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub boot: Id,
    pub revision: u64,
    pub observed_ms: u64,
    pub mode: Mode,
    pub bound_package: Option<Commit>,
    pub selected: Option<ProgramKey>,
    pub loaded: Option<ProgramKey>,
    pub status: Option<Status>,
    pub instance: Option<Instance>,
    pub step: Option<Id>,
    pub elapsed_ms: u64,
    pub owner: Option<Owner>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Select(ProgramKey),
    Load,
    Start { step: Id },
    Pause,
    Resume,
    Next,
    Stop,
    BeginMaintenance,
    CancelMaintenance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request<A = Action> {
    pub lease: Lease,
    pub serial: u64,
    pub expected_revision: u64,
    pub action: A,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt<A = Action, S = State> {
    pub request: Request<A>,
    pub result: Result<(), Code>,
    pub state: S,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameInfo {
    pub boot: Id,
    pub revision: u64,
    pub sampled_ms: u64,
    pub universe: u16,
    pub program: ProgramKey,
    pub instance: Option<Instance>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionAction {
    Start,
    Resume,
    Next,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permission {
    pub package: Identity,
    pub program: ProgramKey,
    pub instance: Instance,
    pub action: PermissionAction,
    pub now_ms: u64,
}
/// Called at the application boundary, never by decoding a peer's "allowed" flag.
/// Time here is monotonic scheduling time, not a trusted wall clock or license implementation.
pub trait PlaybackPolicy {
    /// # Errors
    /// Deny the requested start/resume/advance with a bounded, observable reason.
    fn authorize(&mut self, permission: Permission) -> Result<(), Denial>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    Missing,
    Expired,
    UncertainTime,
    Restricted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Identity,
    Clock,
    Busy,
    Lease,
    Sequence,
    Revision,
    Exhausted,
    Mode,
    Empty,
    Selection,
    NotLoaded,
    Step,
    State,
    Budget,
    Read,
    Integrity,
    Package,
    Allocation,
    Playback,
    Permission(Denial),
}
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identity => "运行身份或租约期限无效",
            Self::Clock => "设备单调时钟不能倒退",
            Self::Busy => "设备或控制权正在使用",
            Self::Lease => "控制权已过期或被接管，请读取当前状态",
            Self::Sequence => "控制请求重复冲突或顺序错误",
            Self::Revision => "操作上下文已改变，请读取当前状态",
            Self::Exhausted => "设备计数已到上限，不能回绕",
            Self::Mode => "当前设备模式不允许此操作",
            Self::Empty => "尚未绑定已安装节目包",
            Self::Selection => "所选节目不在当前包中",
            Self::NotLoaded => "请先载入所选节目",
            Self::Step => "节目中没有所选步骤或下一步",
            Self::State => "当前播放状态不允许此操作",
            Self::Budget => "节目超出当前装载内存预算",
            Self::Read => "节目读取失败，可重试载入",
            Self::Integrity => "节目完整性校验失败",
            Self::Package => "节目格式或能力不支持",
            Self::Allocation => "设备内存不足，未载入节目",
            Self::Playback => "播放核心状态异常",
            Self::Permission(Denial::Missing) => "缺少有效播放许可",
            Self::Permission(Denial::Expired) => "播放许可已到期",
            Self::Permission(Denial::UncertainTime) => "无法确认授权时间",
            Self::Permission(Denial::Restricted) => "播放许可不允许此操作",
        })
    }
}
impl core::error::Error for Code {}
fn package_error(error: &stagemaster_package::Error) -> Code {
    use stagemaster_package::Error;
    match error {
        Error::Read => Code::Read,
        Error::Integrity => Code::Integrity,
        Error::Allocation => Code::Allocation,
        Error::Limit(_) => Code::Budget,
        _ => Code::Package,
    }
}
