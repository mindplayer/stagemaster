//! Application-owned device connectivity. No project, player, storage or DMX dependency.
#![forbid(unsafe_code)]
mod ble;
mod service;
mod transport;
pub use ble::Ble;
use serde::{Deserialize, Serialize};
pub use service::Service;
pub use transport::Transport;

pub const MAX_CANDIDATES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// A discovery handle, NOT authenticated hardware identity.
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Idle,
    Preparing,
    Scanning,
    Connecting,
    Connected,
    Stopping,
    Fault,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub self_test: bool,
    pub output_disabled: bool,
    pub uptime_ms: u32,
    pub ticks: u32,
    pub heap_used: u32,
    pub heap_free: u32,
}
impl From<stagemaster_device_link::client::Diagnostics> for Diagnostics {
    fn from(value: stagemaster_device_link::client::Diagnostics) -> Self {
        Self {
            self_test: value.self_test,
            output_disabled: value.output_disabled,
            uptime_ms: value.uptime_ms,
            ticks: value.ticks,
            heap_used: value.heap_used,
            heap_free: value.heap_free,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub revision: u64,
    pub epoch: u32,
    pub phase: Phase,
    pub candidates: Vec<Candidate>,
    pub scan_performed: bool,
    pub truncated: bool,
    pub selected: Option<Candidate>,
    /// Present only after a successful read, cleared on disconnect/failure.
    pub diagnostics: Option<Diagnostics>,
    pub heartbeat_count: u64,
    pub round_trip_ms: Option<u64>,
    pub last_reply_age_ms: Option<u64>,
    pub problem: Option<Problem>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            revision: 0,
            epoch: 0,
            phase: Phase::Idle,
            candidates: vec![],
            scan_performed: false,
            truncated: false,
            selected: None,
            diagnostics: None,
            heartbeat_count: 0,
            round_trip_ms: None,
            last_reply_age_ms: None,
            problem: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Status,
    Scan { epoch: u32 },
    Connect { epoch: u32, id: String },
    Cancel { epoch: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProblemCode {
    SetupTimeout,
    Permission,
    Adapter,
    PoweredOff,
    Unavailable,
    Timeout,
    Protocol,
    Lost,
    Stale,
    Busy,
    Cleanup,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub code: ProblemCode,
    pub message: String,
    pub detail: Option<String>,
}
impl Problem {
    #[must_use]
    pub fn new(code: ProblemCode) -> Self {
        use ProblemCode as C;
        let message = match code {
            C::SetupTimeout => {
                "系统蓝牙尚未就绪，请先完成蓝牙访问确认；可在系统设置的隐私与安全性中检查蓝牙权限，然后重新搜索"
            }
            C::Permission => "蓝牙访问被拒绝，请在系统设置中允许舞台大师使用蓝牙后重新搜索",
            C::Adapter => "未找到可用蓝牙适配器，请检查系统蓝牙设置后重试",
            C::PoweredOff => "系统蓝牙已关闭，请打开蓝牙后重新搜索",
            C::Unavailable => "设备已不在本次搜索结果中，请重新搜索",
            C::Timeout => "设备未及时回复，请检查供电、距离及是否被其他应用连接，然后重新连接",
            C::Protocol => "设备回复与当前诊断协议不匹配，请核对设备固件",
            C::Lost => "设备连接已中断，请检查供电与距离后重新连接",
            C::Stale => "连接状态已变化，请按当前状态重新操作",
            C::Busy => "正在处理设备连接，请等待完成或取消当前操作",
            C::Cleanup => "系统未确认释放蓝牙资源，请退出应用并检查系统蓝牙后重试",
            C::Closed => "设备连接服务已关闭，请重新打开应用",
        };
        Self {
            code,
            message: message.into(),
            detail: None,
        }
    }
    #[must_use]
    pub fn detail(mut self, mut detail: String) -> Self {
        if let Some((index, _)) = detail.char_indices().nth(512) {
            detail.truncate(index);
        }
        self.detail = Some(detail);
        self
    }
}
impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Problem {}
