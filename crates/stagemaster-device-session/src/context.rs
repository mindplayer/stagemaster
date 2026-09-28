use crate::{Error, HANDSHAKE_MS, LEASE_MS};
const DOMAIN: &[u8; 24] = b"StageMaster/Secure/v1\0\0\0";

/// Public correlation values become authenticated by the Noise transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub connection: u64,
}
impl Context {
    pub(crate) fn encode(self) -> Result<[u8; 64], Error> {
        if self.device == [0; 16] || self.boot == [0; 16] || self.connection == 0 {
            return Err(Error::Invalid);
        }
        let mut bytes = [0; 64];
        bytes[..24].copy_from_slice(DOMAIN);
        bytes[24..40].copy_from_slice(&self.device);
        bytes[40..56].copy_from_slice(&self.boot);
        bytes[56..64].copy_from_slice(&self.connection.to_le_bytes());
        Ok(bytes)
    }
}

pub(crate) struct Clock {
    last: u64,
    deadline: u64,
}
impl Clock {
    pub fn new(now: u64) -> Result<Self, Error> {
        Ok(Self {
            last: now,
            deadline: now.checked_add(HANDSHAKE_MS).ok_or(Error::Clock)?,
        })
    }
    pub fn check(&mut self, now: u64) -> Result<(), Error> {
        if now < self.last {
            return Err(Error::Clock);
        }
        self.last = now;
        if now >= self.deadline {
            return Err(Error::Expired);
        }
        Ok(())
    }
    pub fn received(&mut self, now: u64) -> Result<(), Error> {
        self.establish(now, now)
    }
    pub fn establish(&mut self, received_at: u64, now: u64) -> Result<(), Error> {
        self.check(now)?;
        self.deadline = received_at.checked_add(LEASE_MS).ok_or(Error::Clock)?;
        self.check(now)?;
        Ok(())
    }
}
