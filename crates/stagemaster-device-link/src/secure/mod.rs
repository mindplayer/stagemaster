//! Bounded opaque records for a secure protocol. Framing is not authentication.
mod clock;
mod receiver;
mod record;
mod sender;

pub use receiver::Receiver;
pub use record::Record;
pub use sender::Sender;

pub const MAX_RECORD_BYTES: usize = 1297;
pub const RECORD_HEADER: usize = 4;
pub const RECORD_MS: u64 = 5_000;
const BUFFER_BYTES: usize = MAX_RECORD_BYTES + RECORD_HEADER;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Format,
    Bounds,
    State,
    Clock,
    Expired,
    Closed,
    Fragment(super::management::Error),
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Format => f.write_str("安全记录格式不受支持"),
            Self::Bounds => f.write_str("安全记录或片段超出容量"),
            Self::State => f.write_str("安全记录收发顺序不匹配"),
            Self::Clock => f.write_str("安全记录计时不连续"),
            Self::Expired => f.write_str("安全记录传输超时，请重新连接"),
            Self::Closed => f.write_str("安全记录通道已关闭"),
            Self::Fragment(error) => error.fmt(f),
        }
    }
}
impl core::error::Error for Error {}

fn limit(packet_bytes: usize) -> Result<usize, Error> {
    if (20..=super::management::PACKET_BYTES).contains(&packet_bytes) {
        Ok(packet_bytes - super::management::HEADER_BYTES)
    } else {
        Err(Error::Bounds)
    }
}
