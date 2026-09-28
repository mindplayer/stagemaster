use super::{AUTHENTICATED_LESC, Error, MAX_FRAGMENT, MESSAGE_BYTES, MIN_FRAGMENT, RECEIPT_BYTES};

/// Decode only as correlated peer data. Device-side authority is not constructible here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub diagnostic: u64,
    pub session: [u8; 16],
    pub fragment_bytes: u16,
}
impl Receipt {
    /// # Errors
    /// Rejects empty identifiers and payload budgets outside the version 1 profile.
    pub fn validate(&self) -> Result<(), Error> {
        if self.device == [0; 16]
            || self.boot == [0; 16]
            || self.session == [0; 16]
            || self.diagnostic == 0
        {
            return Err(Error::Identity);
        }
        if !(MIN_FRAGMENT..=MAX_FRAGMENT).contains(&self.fragment_bytes) {
            return Err(Error::Limits);
        }
        Ok(())
    }
    /// # Errors
    /// Invalid data cannot be advertised as a prepared session.
    pub fn encode(&self) -> Result<[u8; RECEIPT_BYTES], Error> {
        self.validate()?;
        let mut out = [0; RECEIPT_BYTES];
        out[..8].copy_from_slice(b"SMAS\x01\x01\x50\0");
        out[8..24].copy_from_slice(&self.device);
        out[24..40].copy_from_slice(&self.boot);
        out[40..48].copy_from_slice(&self.diagnostic.to_le_bytes());
        out[48..64].copy_from_slice(&self.session);
        out[64..66].copy_from_slice(&AUTHENTICATED_LESC.to_le_bytes());
        out[66..68].copy_from_slice(&1_u16.to_le_bytes());
        out[68..70].copy_from_slice(&self.fragment_bytes.to_le_bytes());
        out[70..72].copy_from_slice(&MESSAGE_BYTES.to_le_bytes());
        out[72..76].copy_from_slice(&6000_u32.to_le_bytes());
        Ok(out)
    }
    /// # Errors
    /// Unknown versions, rights, auth methods, reserved bytes and invalid limits fail closed.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != RECEIPT_BYTES
            || &bytes[..8] != b"SMAS\x01\x01\x50\0"
            || bytes[64..68] != [1, 0, 1, 0]
            || bytes[70..72] != MESSAGE_BYTES.to_le_bytes()
            || bytes[72..76] != 6000_u32.to_le_bytes()
            || bytes[76..] != [0; 4]
        {
            return Err(Error::Format);
        }
        let value = Self {
            device: core::array::from_fn(|i| bytes[8 + i]),
            boot: core::array::from_fn(|i| bytes[24 + i]),
            diagnostic: u64::from_le_bytes(core::array::from_fn(|i| bytes[40 + i])),
            session: core::array::from_fn(|i| bytes[48 + i]),
            fragment_bytes: u16::from_le_bytes([bytes[68], bytes[69]]),
        };
        value.validate()?;
        Ok(value)
    }
    /// # Errors
    /// Old boot/connection metadata must never configure a live transmitter.
    pub fn correlate(
        &self,
        device: [u8; 16],
        boot: [u8; 16],
        diagnostic: u64,
    ) -> Result<(), Error> {
        if self.device == device && self.boot == boot && self.diagnostic == diagnostic {
            Ok(())
        } else {
            Err(Error::Identity)
        }
    }
}
