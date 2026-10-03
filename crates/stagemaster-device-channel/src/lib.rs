//! Controller-side application session over bounded records, independent of discovery and BLE.
#![forbid(unsafe_code)]
mod admission;
mod authentication;
mod channel;
mod handshake;
mod messages;
pub mod runtime;
mod runtime_handshake;
mod stream;
pub use channel::Channel;
use std::{future::Future, time::Duration};
pub use stream::StreamRecords;
use tokio::time::Instant;

pub const MAX_RECORD_BYTES: usize = stagemaster_device_session::CIPHERTEXT_BYTES;
pub const RECORD_TIMEOUT: Duration = Duration::from_secs(5);
pub const QUEUED_RECORDS: usize = 4;

#[derive(Debug)]
pub enum Error {
    Closed,
    Timeout,
    Bounds,
    Denied,
    Transport(String),
    Protocol(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => f.write_str("应用记录连接已失效，请重新连接"),
            Self::Timeout => f.write_str("应用记录传输超时，请重新连接"),
            Self::Bounds => f.write_str("应用记录超出容量"),
            Self::Denied => f.write_str("当前设备连接未取得所需应用权限"),
            Self::Transport(detail) | Self::Protocol(detail) => f.write_str(detail),
        }
    }
}
impl std::error::Error for Error {}
pub(crate) fn wire(error: impl std::fmt::Display) -> Error {
    Error::Protocol(error.to_string())
}
pub(crate) fn now(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// One already-established connection, carrying untrusted complete records.
/// Never emits an identity or permission. Bounds are `1..=MAX_RECORD_BYTES` and four queued records.
/// Overflow, I/O error or cancelled send invalidates the connection; no automatic resend.
/// Drop must release receive tasks and local buffers; physical cleanup belongs to the adapter.
pub trait RecordIo: Send + 'static {
    fn send(&mut self, bytes: &[u8]) -> impl Future<Output = Result<(), Error>> + Send;
    /// Nonblocking and cancellation-safe; terminal failure revokes all queued records.
    /// # Errors
    /// Reports overflow, malformed framing or connection failure.
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, Error>;
    fn healthy(&self) -> bool;
    fn close(&mut self);
}

/// Wait for one complete record without cancelling or restarting its partial reader.
/// # Errors
/// Refuses failed, empty, oversized and timed-out records.
pub async fn receive(io: &mut impl RecordIo) -> Result<Vec<u8>, Error> {
    let result = tokio::time::timeout(RECORD_TIMEOUT, async {
        loop {
            if !io.healthy() {
                return Err(Error::Closed);
            }
            if let Some(bytes) = io.try_receive()? {
                return if (1..=MAX_RECORD_BYTES).contains(&bytes.len()) {
                    Ok(bytes)
                } else {
                    Err(Error::Bounds)
                };
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap_or(Err(Error::Timeout));
    if result.is_err() {
        io.close();
    }
    result
}
