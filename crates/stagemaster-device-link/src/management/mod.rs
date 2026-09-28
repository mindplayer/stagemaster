//! Public management wire data, NEVER proof of authentication or a device Grant.
mod packet;
mod receipt;
pub use packet::{Packet, Serial};
pub use receipt::Receipt;

pub const RECEIPT_BYTES: usize = 80;
pub const PACKET_BYTES: usize = 244;
pub const HEADER_BYTES: usize = 4;
pub const MIN_FRAGMENT: u16 = 16;
pub const MAX_FRAGMENT: u16 = 240;
pub const MESSAGE_BYTES: u16 = 1280;
pub const AUTHENTICATED_LESC: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Format,
    Identity,
    Limits,
    Sequence,
    Exhausted,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Format => "设备会话格式不支持",
            Self::Identity => "设备会话与当前连接不一致",
            Self::Limits => "设备会话通信预算无效",
            Self::Sequence => "设备通信片段缺失、重复或乱序",
            Self::Exhausted => "设备片段序号已用尽，请重新连接",
        })
    }
}
impl core::error::Error for Error {}
