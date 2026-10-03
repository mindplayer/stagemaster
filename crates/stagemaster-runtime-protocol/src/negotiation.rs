use crate::{Error, Frame, MAX_MESSAGE_BYTES, VERSION, codec};

/// This declaration is never an authentication grant. Its bits only describe a ready peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    pub observe: bool,
    pub control: bool,
    pub installation: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub min_version: u16,
    pub max_version: u16,
    pub message_bytes: u16,
}
impl Offer {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            min_version: VERSION,
            max_version: VERSION,
            message_bytes: 1280,
        }
    }
    /// # Errors
    /// No common version or insufficient record budget cannot select this protocol.
    pub fn select(&self) -> Result<u16, Error> {
        self.validate()?;
        if !(self.min_version..=self.max_version).contains(&VERSION) {
            return Err(Error::Version);
        }
        if usize::from(self.message_bytes) != MAX_MESSAGE_BYTES {
            return Err(Error::Bounds);
        }
        Ok(VERSION)
    }
    pub(crate) fn validate(self) -> Result<(), Error> {
        if self.min_version == 0 || self.min_version > self.max_version {
            return Err(Error::Version);
        }
        if self.message_bytes == 0 || usize::from(self.message_bytes) > MAX_MESSAGE_BYTES {
            return Err(Error::Bounds);
        }
        Ok(())
    }
    /// # Errors
    /// Reject invalid version ranges and unsupported budgets.
    pub fn encode(&self) -> Result<Frame, Error> {
        codec::encode(0, |e| codec::negotiation::offer(e, *self))
    }
    /// # Errors
    /// Refuse malformed or unknown bootstrap envelopes.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        codec::decode(bytes, 0, codec::negotiation::read_offer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub connection: u64,
    pub session: [u8; 16],
    pub principal: [u8; 16],
    pub permission_revision: u64,
}
impl Peer {
    pub(crate) fn validate(self) -> Result<(), Error> {
        for id in [self.device, self.boot, self.session, self.principal] {
            codec::valid_id(id)?;
        }
        if self.connection == 0 || self.permission_revision == 0 {
            return Err(Error::Identity);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ready {
    pub peer: Peer,
    pub version: u16,
    pub message_bytes: u16,
    pub access: Access,
    pub remaining_ms: u32,
}
impl Ready {
    pub(crate) fn validate(self) -> Result<(), Error> {
        self.peer.validate()?;
        if self.version != VERSION {
            return Err(Error::Version);
        }
        if usize::from(self.message_bytes) != MAX_MESSAGE_BYTES
            || !(1..=600_000).contains(&self.remaining_ms)
        {
            return Err(Error::Bounds);
        }
        if !self.access.observe && !self.access.control {
            return Err(Error::Identity);
        }
        Ok(())
    }
    /// Call only after decrypting the whole ready message in the current confirmed session.
    /// `expected` comes from trusted provisioning and this exact secure handshake.
    /// # Errors
    /// Reject mismatched identity/rights, unsupported selection and a longer permission.
    pub fn correlate(&self, offered: Offer, expected: &Self) -> Result<(), Error> {
        self.validate()?;
        expected.validate()?;
        if self.version != offered.select()?
            || self.peer != expected.peer
            || self.access != expected.access
            || self.remaining_ms > expected.remaining_ms
        {
            return Err(Error::Correlation);
        }
        Ok(())
    }
    /// # Errors
    /// Only complete valid runtime declarations may be encoded.
    pub fn encode(&self) -> Result<Frame, Error> {
        codec::encode(1, |e| codec::negotiation::ready(e, *self))
    }
    /// # Errors
    /// Unknown fields, scope bits, identities and budgets are refused.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        codec::decode(bytes, 1, codec::negotiation::read_ready)
    }
}
