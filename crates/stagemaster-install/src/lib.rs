//! Transport-independent package installation. Installing never starts playback.
#![no_std]
#![forbid(unsafe_code)]
mod installer;
mod record;
use core::fmt;
pub use installer::{Installed, Installer, RecoveryReport, SlotHealth};
pub use record::{Commit, RECORD_BYTES};
use stagemaster_package::{Archive, MAX_PACKAGE_BYTES, ReadAt};

pub const MAX_CHUNK_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    A,
    B,
}
impl Slot {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub bytes: usize,
    pub digest: [u8; 32],
}
impl Identity {
    #[must_use]
    pub const fn from_archive(archive: &Archive) -> Self {
        Self {
            bytes: archive.total_bytes(),
            digest: *archive.digest(),
        }
    }
    pub(crate) fn validate(self) -> Result<(), Code> {
        if !(64..=MAX_PACKAGE_BYTES).contains(&self.bytes) {
            return Err(Code::Bounds);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub boot: [u8; 16],
    pub counter: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Receiving,
    Verified,
    Committed,
    Cancelled,
    Failed,
    Uncertain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub transaction: Transaction,
    pub identity: Identity,
    pub received: usize,
    pub phase: Phase,
    /// Planned destination; a durable receipt only when phase is Committed.
    pub commit: Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Identity,
    Stale,
    Order,
    Busy,
    Bounds,
    Conflict,
    Incomplete,
    State,
    Uncertain,
    Empty,
    Exhausted,
    Metadata,
}
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identity => "安装启动身份无效",
            Self::Stale => "安装事务已过期，请重新读取设备状态",
            Self::Order => "事务计数或接收偏移不连续",
            Self::Busy => "已有安装事务尚未结束",
            Self::Bounds => "包长度或分块范围超出限制",
            Self::Conflict => "重复请求的内容不一致",
            Self::Incomplete => "播放包尚未接收完整",
            Self::State => "当前安装状态不允许此操作",
            Self::Uncertain => "提交结果待确认，请先重新核对存储状态",
            Self::Empty => "尚无有效的已安装播放包",
            Self::Exhausted => "安装计数已达上限，不能回绕",
            Self::Metadata => "安装记录无效或代数冲突",
        })
    }
}
impl core::error::Error for Code {}
#[derive(Debug)]
pub enum Error<E> {
    Code(Code),
    Storage(E),
    Package(stagemaster_package::Error),
    CommitUncertain(E),
}
impl<E> From<Code> for Error<E> {
    fn from(code: Code) -> Self {
        Self::Code(code)
    }
}
impl<E> From<stagemaster_package::Error> for Error<E> {
    fn from(error: stagemaster_package::Error) -> Self {
        Self::Package(error)
    }
}
impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(e) => e.fmt(f),
            Self::Storage(e) => write!(f, "安装存储失败：{e}"),
            Self::Package(e) => e.fmt(f),
            Self::CommitUncertain(e) => write!(f, "提交结果待确认：{e}"),
        }
    }
}
impl<E: fmt::Debug + fmt::Display> core::error::Error for Error<E> {}

pub enum Record {
    Absent,
    Invalid,
    Bytes([u8; RECORD_BYTES]),
}

/// Exclusive installer storage. All mutation is serialized through one Installer.
/// A successful commit is durable. An error may be post-commit; settle + reread is required.
/// Snapshots own an immutable slot lease, including across reopening this storage.
pub trait Storage {
    type Error;
    type Snapshot: ReadAt;
    fn capacity(&self, slot: Slot) -> usize;
    /// # Errors
    /// I/O failure must not be translated into a missing record.
    fn record(&self, slot: Slot) -> Result<Record, Self::Error>;
    /// # Errors
    /// Return the actual slot length; reject missing/inaccessible payloads.
    fn slot_len(&self, slot: Slot) -> Result<usize, Self::Error>;
    /// # Errors
    /// No short reads or silent zero padding.
    fn read(&self, slot: Slot, offset: usize, target: &mut [u8]) -> Result<(), Self::Error>;
    /// # Errors
    /// Refuse pinned slots. Failure may leave scratch bytes but never changes the other slot.
    fn prepare(&mut self, slot: Slot, bytes: usize) -> Result<(), Self::Error>;
    /// # Errors
    /// Failure may leave a partial block; the transaction must then be abandoned.
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> Result<(), Self::Error>;
    /// # Errors
    /// On success the entire staged payload is durable before metadata publication.
    fn sync_payload(&mut self, slot: Slot) -> Result<(), Self::Error>;
    /// # Errors
    /// Publish a durable record without touching the other slot. Failure may leave
    /// the target record old, new, or detectably damaged; reconciliation must decide.
    fn commit_record(&mut self, commit: Commit) -> Result<(), Self::Error>;
    /// # Errors
    /// Establish persistence before reconciling a possibly completed commit.
    fn settle(&mut self) -> Result<(), Self::Error>;
    /// Release staging resources, never remove or rewrite committed records.
    fn release(&mut self);
    /// # Errors
    /// Acquire a stable immutable read lease; refuse an exclusively reserved slot.
    fn snapshot(&self, slot: Slot) -> Result<Self::Snapshot, Self::Error>;
}
