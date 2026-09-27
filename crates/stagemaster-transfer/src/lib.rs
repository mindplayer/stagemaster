//! Bounded software installation protocol; no radio, authentication or playback control.
#![no_std]
#![forbid(unsafe_code)]
mod codec;
mod framing;
mod service;
mod upload;

use core::fmt;
pub use framing::Assembler;
pub use service::{AuthorizedLink, Service};
use stagemaster_install::{Commit, Identity, Progress, Transaction};
pub use upload::{Outcome, Upload, UploadError};

pub const MAX_FRAME_BYTES: usize = 1280;
pub const GROUP: u16 = 0x5354;
pub const VERSION: u8 = 1;
pub type Id = [u8; 16];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    bytes: [u8; MAX_FRAME_BYTES],
    length: usize,
}
impl Frame {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    const fn empty() -> Self {
        Self {
            bytes: [0; MAX_FRAME_BYTES],
            length: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Malformed,
    Version,
    Bounds,
    Sequence,
    Connection,
    Denied,
    Busy,
    Exhausted,
    State,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Malformed => "安装消息格式不正确",
            Self::Version => "安装协议版本或命令不支持",
            Self::Bounds => "安装消息或字段超出范围",
            Self::Sequence => "安装请求重复冲突或顺序不正确",
            Self::Connection => "安装连接已失效，请重新连接并核对状态",
            Self::Denied => "尚未授予安装连接权限",
            Self::Busy => "已有安装连接或事务正在使用",
            Self::Exhausted => "安装请求序号已达上限",
            Self::State => "安装协议状态不一致",
        })
    }
}
impl core::error::Error for Error {}
impl From<minicbor::decode::Error> for Error {
    fn from(_: minicbor::decode::Error) -> Self {
        Self::Malformed
    }
}
impl From<minicbor::encode::Error<minicbor::encode::write::EndOfSlice>> for Error {
    fn from(_: minicbor::encode::Error<minicbor::encode::write::EndOfSlice>) -> Self {
        Self::Bounds
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    Status = 0,
    Begin = 1,
    Write = 2,
    Verify = 3,
    Commit = 4,
    Cancel = 5,
    Reconcile = 6,
}
impl Command {
    fn read(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::Status),
            1 => Ok(Self::Begin),
            2 => Ok(Self::Write),
            3 => Ok(Self::Verify),
            4 => Ok(Self::Commit),
            5 => Ok(Self::Cancel),
            6 => Ok(Self::Reconcile),
            _ => Err(Error::Version),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action<'a> {
    Status,
    Begin {
        transaction: Transaction,
        identity: Identity,
    },
    Write {
        transaction: Transaction,
        offset: usize,
        bytes: &'a [u8],
    },
    Verify(Transaction),
    Commit(Transaction),
    Cancel(Transaction),
    Reconcile(Transaction),
}
impl Action<'_> {
    #[must_use]
    pub const fn command(self) -> Command {
        match self {
            Self::Status => Command::Status,
            Self::Begin { .. } => Command::Begin,
            Self::Write { .. } => Command::Write,
            Self::Verify(_) => Command::Verify,
            Self::Commit(_) => Command::Commit,
            Self::Cancel(_) => Command::Cancel,
            Self::Reconcile(_) => Command::Reconcile,
        }
    }
    const fn transaction(self) -> Option<Transaction> {
        match self {
            Self::Status => None,
            Self::Begin { transaction, .. }
            | Self::Write { transaction, .. }
            | Self::Verify(transaction)
            | Self::Commit(transaction)
            | Self::Cancel(transaction)
            | Self::Reconcile(transaction) => Some(transaction),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request<'a> {
    pub link: Id,
    pub id: u64,
    pub action: Action<'a>,
}
impl<'a> Request<'a> {
    /// # Errors
    /// Reject invalid fields or messages exceeding the fixed wire limit.
    pub fn encode(self) -> Result<Frame, Error> {
        codec::encode_request(self)
    }
    /// # Errors
    /// Strictly decode this application group, including complete payload consumption.
    pub fn decode(bytes: &'a [u8]) -> Result<Self, Error> {
        codec::decode_request(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub boot: Id,
    pub head: Option<Commit>,
    pub progress: Option<Progress>,
    /// Whether this authorized principal owns the reported transaction.
    pub owned: bool,
    pub max_chunk: usize,
    /// Protocol package ceiling, not a claim of physical free flash space.
    pub max_package: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RemoteError {
    Identity = 1,
    Stale = 2,
    Order = 3,
    Busy = 4,
    Bounds = 5,
    Conflict = 6,
    Incomplete = 7,
    State = 8,
    Uncertain = 9,
    Empty = 10,
    Exhausted = 11,
    Metadata = 12,
    Storage = 30,
    Package = 31,
    Ownership = 32,
}
impl RemoteError {
    fn read(value: u8) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::Identity),
            2 => Ok(Self::Stale),
            3 => Ok(Self::Order),
            4 => Ok(Self::Busy),
            5 => Ok(Self::Bounds),
            6 => Ok(Self::Conflict),
            7 => Ok(Self::Incomplete),
            8 => Ok(Self::State),
            9 => Ok(Self::Uncertain),
            10 => Ok(Self::Empty),
            11 => Ok(Self::Exhausted),
            12 => Ok(Self::Metadata),
            30 => Ok(Self::Storage),
            31 => Ok(Self::Package),
            32 => Ok(Self::Ownership),
            _ => Err(Error::Version),
        }
    }
}
impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Identity => "安装启动身份无效",
            Self::Stale => "设备安装事务已过期",
            Self::Order => "设备要求连续的事务计数与分块偏移",
            Self::Busy => "设备已有未完成安装",
            Self::Bounds => "超出设备安装限制",
            Self::Conflict => "重复安装内容不一致",
            Self::Incomplete => "设备尚未收到完整播放包",
            Self::State => "设备安装状态不允许此操作",
            Self::Uncertain => "设备提交结果待确认",
            Self::Empty => "设备没有已安装节目",
            Self::Exhausted => "设备安装计数已耗尽",
            Self::Metadata => "设备安装记录冲突",
            Self::Storage => "设备安装存储失败",
            Self::Package => "设备拒绝无效或不兼容的播放包",
            Self::Ownership => "此安装属于另一操作主体",
        })
    }
}
impl core::error::Error for RemoteError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Response {
    pub link: Id,
    pub id: u64,
    pub command: Command,
    pub result: Result<(), RemoteError>,
    pub state: State,
}
impl Response {
    /// # Errors
    /// Reject invalid response fields or capacity overflow.
    pub fn encode(&self) -> Result<Frame, Error> {
        codec::encode_response(self)
    }
    /// # Errors
    /// Reject unknown fields, versions, malformed records and inconsistent states.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        codec::decode_response(bytes)
    }
}
