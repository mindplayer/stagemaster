//! Single owner of live device-management authority. No wire or hardware APIs.
mod connection;
mod pairing;
#[cfg(test)]
mod tests;
mod types;
use crate::{LocalIdentity, Vault};
pub use types::{Connection, Error, Evidence, Grant, Origin, Peer, Security};

pub const ADMISSION_MS: u64 = 90_000;
pub const LEASE_MS: u64 = 6_000;
pub const PAIRING_ATTEMPTS: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Awaiting,
    Pairing,
    Granted([u8; 16]),
}
struct Active {
    id: Connection,
    session: [u8; 16],
    deadline: u64,
    phase: Phase,
}
struct Window {
    deadline: u64,
    attempts: u8,
}

pub struct Authority {
    vault: Option<Vault>,
    local: LocalIdentity,
    confirmed_generation: u64,
    next_epoch: u32,
    previous_session: Option<[u8; 16]>,
    last_ms: u64,
    active: Option<Active>,
    window: Option<Window>,
}
impl Authority {
    /// Only pass a snapshot verified by the storage owner, never an uncommitted proposal.
    #[must_use]
    pub fn new(verified: Vault, now_ms: u64) -> Self {
        Self {
            local: verified.local().clone(),
            confirmed_generation: verified.generation(),
            vault: Some(verified),
            next_epoch: 0,
            previous_session: None,
            last_ms: now_ms,
            active: None,
            window: None,
        }
    }
    #[must_use]
    pub const fn vault(&self) -> Option<&Vault> {
        self.vault.as_ref()
    }

    /// Immediately revoke all runtime authority before storage mutation or recovery.
    /// Caller must also revoke worker epochs and disconnect the physical link.
    pub fn suspend(&mut self) -> Option<Connection> {
        self.vault = None;
        self.window = None;
        self.active.take().map(|active| active.id)
    }
    /// Restore only a verified durable outcome; never restores a prior connection.
    /// # Errors
    /// Refuses active snapshots, backwards confirmed revisions and another device identity.
    pub fn restore(&mut self, verified: Vault) -> Result<(), Error> {
        if self.vault.is_some() {
            return Err(Error::Busy);
        }
        if verified.local() != &self.local || verified.generation() < self.confirmed_generation {
            return Err(crate::Code::Conflict.into());
        }
        self.confirmed_generation = verified.generation();
        self.vault = Some(verified);
        Ok(())
    }
    /// Poll even when idle. Returns the link that the adapter must disconnect.
    /// # Errors
    /// A backwards clock closes the connection and pairing window, without rearming either.
    pub fn poll(&mut self, now_ms: u64) -> Result<Option<Connection>, Error> {
        if now_ms < self.last_ms {
            self.active = None;
            self.window = None;
            return Err(Error::Clock);
        }
        self.last_ms = now_ms;
        if self.window.as_ref().is_some_and(|w| now_ms >= w.deadline) {
            self.window = None;
        }
        if self.active.as_ref().is_some_and(|a| {
            now_ms >= a.deadline || (a.phase == Phase::Pairing && self.window.is_none())
        }) {
            return Ok(self.active.take().map(|active| active.id));
        }
        Ok(None)
    }
    /// Stale events never close a newer physical connection.
    pub fn disconnect(&mut self, connection: Connection) {
        if self.active.as_ref().is_some_and(|a| a.id == connection) {
            self.active = None;
        }
    }
    fn check(&mut self, connection: Connection, now_ms: u64) -> Result<(), Error> {
        let active = self.active.as_ref().ok_or(Error::Closed)?;
        if active.id != connection {
            return Err(Error::Stale);
        }
        if self.poll(now_ms)?.is_some() {
            return Err(Error::Expired);
        }
        if self.vault.is_none() {
            return Err(Error::Unavailable);
        }
        Ok(())
    }
    fn close<T>(&mut self, error: Error) -> Result<T, Error> {
        self.active = None;
        Err(error)
    }
    fn deadline(&mut self, now_ms: u64, duration: u64) -> Result<u64, Error> {
        match now_ms.checked_add(duration) {
            Some(deadline) => Ok(deadline),
            None => self.close(Error::Exhausted),
        }
    }
}
