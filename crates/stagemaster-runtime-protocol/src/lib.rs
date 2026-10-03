//! Bounded device runtime messages, not authentication, scheduling or physical output.
#![no_std]
#![forbid(unsafe_code)]
mod codec;
mod model;
mod negotiation;
mod response;
pub use model::{Detail, Failure, Operation, Program, Reply, Request, Step, Text};
pub use negotiation::{Access, Offer, Peer, Ready};
pub use response::{Body, Observation, Response};

pub const VERSION: u16 = 1;
pub const MAX_MESSAGE_BYTES: usize = 1280;

/// One complete application message, before authenticated record encryption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    bytes: [u8; MAX_MESSAGE_BYTES],
    length: usize,
}
impl Frame {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Format,
    Version,
    Bounds,
    Identity,
    Correlation,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Format => "设备运行消息格式无效",
            Self::Version => "设备运行协议版本不兼容",
            Self::Bounds => "设备运行消息超出容量",
            Self::Identity => "设备运行消息身份无效",
            Self::Correlation => "设备运行回复与当前请求或连接不匹配",
        })
    }
}
impl core::error::Error for Error {}
impl From<minicbor::decode::Error> for Error {
    fn from(_: minicbor::decode::Error) -> Self {
        Self::Format
    }
}
impl<E> From<minicbor::encode::Error<E>> for Error {
    fn from(_: minicbor::encode::Error<E>) -> Self {
        Self::Bounds
    }
}

impl Request {
    /// # Errors
    /// Reject invalid identities, unsupported values or message overflow.
    pub fn encode(&self) -> Result<Frame, Error> {
        codec::encode(2, |e| codec::request::write(e, *self))
    }
    /// # Errors
    /// Strictly rejects unknown versions, invalid fields, truncation and trailing data.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        codec::decode(bytes, 2, codec::request::read)
    }
}
