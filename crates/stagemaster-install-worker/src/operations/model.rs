use stagemaster_package::{Id, MAX_TEXT_BYTES};
use stagemaster_runtime::{Action, Code, ProgramKey, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Status,
    Catalog { index: u16 },
    Step { index: u16 },
    Acquire { duration_ms: u64, takeover: bool },
    Renew { duration_ms: u64 },
    Release,
    Apply(Action),
    FinishMaintenance,
}

/// Internal application request, not a wire credential. A future decoder must
/// correlate the session with the authenticated connection before queueing it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub session: Id,
    pub id: u64,
    pub expected_revision: u64,
    pub operation: Operation,
}

/// Owned bounded text; never borrows a catalogue which later load/maintenance can drop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Text {
    bytes: [u8; MAX_TEXT_BYTES],
    len: usize,
}
impl Text {
    pub(super) fn copy(value: &str) -> Result<Self, Failure> {
        if value.len() > MAX_TEXT_BYTES {
            return Err(Failure::Bounds);
        }
        let mut text = Self {
            bytes: [0; MAX_TEXT_BYTES],
            len: value.len(),
        };
        text.bytes[..value.len()].copy_from_slice(value.as_bytes());
        Ok(text)
    }
    /// # Panics
    /// Only if an implementation bug violates this type's private UTF-8 invariant.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // Only copy() constructs this value, from valid UTF-8 without truncation.
        core::str::from_utf8(&self.bytes[..self.len]).expect("bounded UTF-8 text")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Program {
    pub key: ProgramKey,
    pub name: Text,
    pub loader_bytes: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub id: Id,
    pub name: Text,
    pub number: Text,
}

// Exactly one bounded response per connection, no allocator or growing catalogue queue.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    State,
    Program(Option<Program>),
    Step(Option<Step>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Runtime(Code),
    Storage,
    Bounds,
}
impl core::fmt::Display for Failure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Runtime(code) => code.fmt(f),
            Self::Storage => f.write_str("设备节目读取或校验失败，可重试载入"),
            Self::Bounds => f.write_str("设备运行回复超出容量范围"),
        }
    }
}
impl core::error::Error for Failure {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub request: Request,
    pub result: Result<Detail, Failure>,
    /// Historical observation at this request; never a physical output acknowledgement.
    pub state: State,
    pub program_count: u16,
    pub step_count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Closed,
    Obsolete,
    Identity,
    Denied,
    Sequence,
    Clock,
    Runtime(Code),
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Closed => f.write_str("设备运行连接已关闭"),
            Self::Obsolete => f.write_str("设备操作权限已失效，请重新连接并读取状态"),
            Self::Identity => f.write_str("设备运行请求不属于当前连接或启动"),
            Self::Denied => f.write_str("当前连接没有此设备操作权限"),
            Self::Sequence => f.write_str("设备运行请求序号冲突，请重新读取状态"),
            Self::Clock => f.write_str("设备运行入口计时异常"),
            Self::Runtime(error) => error.fmt(f),
        }
    }
}
impl core::error::Error for Error {}
