use super::Error;
use stagemaster_device_session::{Context, PeerProof};

pub const MAX_PERMISSION_MS: u32 = 600_000;

/// Explicit trusted development configuration, NOT a peer-supplied credential.
/// There is deliberately no decoder or production/cloud origin switch.
#[derive(Clone, Copy)]
pub struct DevelopmentPermit {
    device: [u8; 16],
    holder: [u8; 32],
    principal: [u8; 16],
    revision: u64,
    duration_ms: u32,
}
impl DevelopmentPermit {
    /// # Errors
    /// Empty identities, revision zero and unbounded durations are refused.
    pub fn installation(
        device: [u8; 16],
        holder: [u8; 32],
        principal: [u8; 16],
        revision: u64,
        duration_ms: u32,
    ) -> Result<Self, Error> {
        if device == [0; 16]
            || holder == [0; 32]
            || principal == [0; 16]
            || revision == 0
            || !(1..=MAX_PERMISSION_MS).contains(&duration_ms)
        {
            return Err(Error::Invalid);
        }
        Ok(Self {
            device,
            holder,
            principal,
            revision,
            duration_ms,
        })
    }
    pub(super) fn admit(self, peer: PeerProof, context: Context, now: u64) -> Result<Grant, Error> {
        if peer.context() != context
            || context.device != self.device
            || *peer.public_key() != self.holder
            || peer.session() == [0; 16]
        {
            return Err(Error::Denied);
        }
        Ok(Grant {
            context,
            principal: self.principal,
            session: peer.session(),
            revision: self.revision,
            until: now
                .checked_add(u64::from(self.duration_ms))
                .ok_or(Error::Clock)?,
        })
    }
}

/// A snapshot, not a transferable bearer token. Only Session can construct it;
/// dispatch must recheck the Session and publish worker revocation independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    pub(super) context: Context,
    pub(super) principal: [u8; 16],
    pub(super) session: [u8; 16],
    pub(super) revision: u64,
    pub(super) until: u64,
}
impl Grant {
    #[must_use]
    pub const fn context(self) -> Context {
        self.context
    }
    #[must_use]
    pub const fn principal(self) -> [u8; 16] {
        self.principal
    }
    #[must_use]
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }
    #[must_use]
    pub const fn expires_at(self) -> u64 {
        self.until
    }
}
