//! Authenticated records to the existing worker. No radio, files or cloud client.
mod outgoing;
use crate::{Command, Completion, Endpoint, Epoch, Phase};
use stagemaster_device_auth::application::{Grant, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, MAX_PAYLOAD, PLAINTEXT_BYTES};
use stagemaster_transfer::AuthorizedLink;

const _: () = assert!(MAX_PAYLOAD == stagemaster_transfer::MAX_FRAME_BYTES);
const _: () = assert!(MAX_PAYLOAD == stagemaster_device_link::management::MESSAGE_BYTES as usize);
const _: () = assert!(
    stagemaster_device_session::AUTHENTICATION
        == stagemaster_device_link::management::AUTHENTICATED_APPLICATION
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Permission(stagemaster_device_auth::application::Error),
    Endpoint(crate::ChannelError),
    Receipt(stagemaster_device_link::management::Error),
    Order,
}
impl From<stagemaster_device_auth::application::Error> for Error {
    fn from(value: stagemaster_device_auth::application::Error) -> Self {
        Self::Permission(value)
    }
}
impl From<crate::ChannelError> for Error {
    fn from(value: crate::ChannelError) -> Self {
        Self::Endpoint(value)
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Permission(value) => value.fmt(f),
            Self::Endpoint(value) => value.fmt(f),
            Self::Receipt(value) => value.fmt(f),
            Self::Order => f.write_str("加密安装通信顺序异常，请重新连接"),
        }
    }
}
impl core::error::Error for Error {}

#[derive(Clone, Copy)]
enum Sending {
    Ready,
    Reply,
    Heartbeat,
}

/// One connection and one immutable ciphertext in flight. Adapter errors must call
/// close and clear the independent worker epoch, including on cancellation/drop.
pub struct Gateway {
    access: Session,
    endpoint: Endpoint,
    ready: bool,
    heartbeat: bool,
    sending: Option<Sending>,
    out: [u8; CIPHERTEXT_BYTES],
    length: usize,
}
impl Gateway {
    /// Return Open only after live admission. Queue failure requires `close()`.
    /// # Errors
    /// Reject expired permissions or an invalid local worker epoch/link.
    pub fn open(mut access: Session, epoch: Epoch, now: u64) -> Result<(Self, Command), Error> {
        let grant = access.grant(now)?;
        let (endpoint, command) = Endpoint::open(
            epoch,
            AuthorizedLink {
                principal: grant.principal(),
                session: grant.session(),
            },
            MAX_PAYLOAD,
            now,
        )?;
        Ok((
            Self {
                access,
                endpoint,
                ready: false,
                heartbeat: false,
                sending: None,
                out: [0; CIPHERTEXT_BYTES],
                length: 0,
            },
            command,
        ))
    }
    pub fn close(&mut self) {
        self.access.revoke();
        self.endpoint.close();
        self.ready = false;
        self.heartbeat = false;
        self.sending = None;
        self.out.fill(0);
        self.length = 0;
    }
    /// Time-check before publishing to the worker; never cache a successful result.
    pub fn live_epoch(&mut self, now: u64) -> Option<Epoch> {
        self.poll(now).ok()?;
        self.endpoint.live_epoch()
    }
    /// # Errors
    /// Permission/security and worker deadlines revoke the whole gateway.
    pub fn poll(&mut self, now: u64) -> Result<Grant, Error> {
        let result = (|| {
            let grant = self.access.grant(now)?;
            self.endpoint.poll(now)?;
            Ok(grant)
        })();
        self.checked(result)
    }
    /// Accept an encrypted heartbeat or a complete SMP request, never a partial one.
    /// # Errors
    /// Unsolicited messages, multiple pending heartbeats or any decode error close.
    pub fn receive(&mut self, cipher: &[u8], now: u64) -> Result<Option<Command>, Error> {
        self.poll(now)?;
        let mut plaintext = [0; PLAINTEXT_BYTES];
        let result = (|| {
            let record = self.access.open(cipher, &mut plaintext, now)?;
            match record.kind {
                Kind::Heartbeat if !self.heartbeat => {
                    self.heartbeat = true;
                    Ok(None)
                }
                Kind::Message if self.ready => self
                    .endpoint
                    .receive(record.payload, now)?
                    .map(Some)
                    .ok_or(Error::Order),
                _ => Err(Error::Order),
            }
        })();
        plaintext.fill(0);
        self.checked(result)
    }
    /// # Errors
    /// Check authority before and after work; late completions cannot restore it.
    pub fn complete(&mut self, completion: Completion, now: u64) -> Result<bool, Error> {
        self.poll(now)?;
        let result = self.endpoint.complete(completion, now).map_err(Error::from);
        self.checked(result)
    }
    fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
}
impl Drop for Gateway {
    fn drop(&mut self) {
        self.close();
    }
}
