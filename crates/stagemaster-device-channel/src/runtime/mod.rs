//! One serialized runtime client; scheduling and periodic heartbeat ownership stay with the host.
mod incoming;
mod requests;
use crate::{Channel, Error, RecordIo};
use stagemaster_runtime_protocol::{Ready, Request, Response};
use std::time::Duration;
use tokio::time::Instant;

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
pub struct RuntimeClient<R: RecordIo> {
    channel: Channel<R>,
    serial: u64,
    pending: Option<(Request, Instant)>,
    last: Option<Response>,
}
impl<R: RecordIo> RuntimeClient<R> {
    /// # Errors
    /// Only a currently usable, explicitly negotiated runtime channel is accepted.
    pub fn new(mut channel: Channel<R>) -> Result<Self, Error> {
        if channel.runtime_peer().is_none()
            || channel.application_started
            || channel.pending.is_some()
        {
            channel.close();
            return Err(Error::Denied);
        }
        Ok(Self {
            channel,
            serial: 0,
            pending: None,
            last: None,
        })
    }
    #[must_use]
    pub fn peer(&self) -> Option<Ready> {
        if self.expired() {
            None
        } else {
            self.channel.runtime_peer()
        }
    }
    /// Remains available after errors/close to explain uncertain execution. Never auto-replay it.
    #[must_use]
    pub fn pending(&self) -> Option<Request> {
        self.pending.map(|(request, _)| request)
    }
    /// Historical observation only. It does not prove the peer is online or physically outputting.
    #[must_use]
    pub const fn last_response(&self) -> Option<&Response> {
        self.last.as_ref()
    }
    /// Closing the input channel sends neither Stop nor an implicit retry.
    pub fn close(&mut self) {
        self.channel.close();
    }
    /// Keepalive while idle or awaiting a reply. Does not renew permission or request deadlines.
    /// # Errors
    /// Stale, cancelled or failed heartbeat invalidates this connection.
    pub async fn heartbeat(&mut self) -> Result<(), Error> {
        self.check()?;
        let result = self.channel.heartbeat().await;
        self.checked(result)?;
        self.check()
    }
    fn expired(&self) -> bool {
        self.pending
            .is_some_and(|(_, began)| began.elapsed() >= REQUEST_TIMEOUT)
    }
    fn check(&mut self) -> Result<(), Error> {
        let result = if self.expired() {
            Err(Error::Timeout)
        } else {
            self.channel.check()
        };
        self.checked(result)
    }
    fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
}
