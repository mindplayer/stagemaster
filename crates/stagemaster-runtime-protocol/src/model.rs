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
    /// Copy bounded UTF-8 without truncation or allocation.
    /// # Errors
    /// Reject text longer than the package label limit.
    pub fn new(value: &str) -> Result<Self, Failure> {
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
        // Only new() constructs this value, from valid UTF-8 without truncation.
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

impl From<Code> for Failure {
    fn from(value: Code) -> Self {
        Self::Runtime(value)
    }
}
