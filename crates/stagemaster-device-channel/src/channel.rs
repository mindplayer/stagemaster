use crate::{Error, RecordIo, admission::Admission, now, wire};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::Channel as Secure;
use tokio::time::Instant;

/// Owns one confirmed application session. No mutable access to its transport or keys.
pub struct Channel<R: RecordIo> {
    pub(crate) admission: Admission,
    pub(crate) secure: Secure,
    pub(crate) io: R,
    pub(crate) origin: Instant,
    pub(crate) received_at: u64,
    pub(crate) until: u64,
    pub(crate) pending: Option<Vec<u8>>,
    pub(crate) usable: bool,
    pub(crate) application_started: bool,
}
impl<R: RecordIo> Channel<R> {
    /// Returns authenticated installation facts only while this exact connection is usable.
    /// Receipt fields retain their original values; `remaining_ms` is not a refreshed lease.
    #[must_use]
    pub fn peer(&self) -> Option<ApplicationReceipt> {
        match self.admission {
            Admission::Installation(receipt) if self.alive() => Some(receipt),
            _ => None,
        }
    }
    /// Runtime readiness is separate from installation; values remain historical.
    #[must_use]
    pub fn runtime_peer(&self) -> Option<stagemaster_runtime_protocol::Ready> {
        match self.admission {
            Admission::Runtime(receipt) if self.alive() => Some(receipt),
            _ => None,
        }
    }
    pub(crate) fn alive(&self) -> bool {
        let time = now(self.origin);
        self.usable
            && self.io.healthy()
            && time < self.until
            && time.saturating_sub(self.received_at) < stagemaster_device_session::LEASE_MS
    }
    /// # Errors
    /// Public declarations must still match the authenticated connection.
    pub fn correlate(&self, bytes: &[u8]) -> Result<(), Error> {
        let desc = stagemaster_device_info::Description::decode(bytes).map_err(wire)?;
        if !self.alive() || !self.admission.matches(desc) {
            return Err(Error::Denied);
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.usable = false;
        self.secure.close();
        self.pending = None;
        self.io.close();
    }
    pub(crate) fn check(&mut self) -> Result<(), Error> {
        if !self.alive() {
            self.close();
            return Err(Error::Closed);
        }
        let result = self.secure.poll(now(self.origin)).map_err(wire);
        self.checked(result)
    }
    pub(crate) fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
    pub(crate) fn begin(&mut self) -> Result<(), Error> {
        self.check()?;
        // Dropping an in-flight future cannot restore this connection or reuse its nonce.
        self.usable = false;
        Ok(())
    }
    pub(crate) fn finish(&mut self, result: Result<(), Error>) -> Result<(), Error> {
        self.checked(result)?;
        self.usable = true;
        self.check()
    }
}
impl<R: RecordIo> Drop for Channel<R> {
    fn drop(&mut self) {
        self.close();
    }
}
