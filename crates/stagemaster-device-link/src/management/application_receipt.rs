use super::{AUTHENTICATED_APPLICATION, Error, MESSAGE_BYTES};

pub const APPLICATION_RECEIPT_BYTES: usize = 112;
const HEADER: &[u8; 8] = b"SMAP\x01\x01\x70\0";

/// Data carried ONLY inside the confirmed encrypted channel, after worker Opened.
/// Decoding this type never creates authority. Version 1 grants installation only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationReceipt {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub diagnostic: u64,
    pub session: [u8; 16],
    pub principal: [u8; 16],
    pub revision: u64,
    pub remaining_ms: u32,
}
impl ApplicationReceipt {
    /// # Errors
    /// Empty identities and invalid development permission budgets are refused.
    pub fn validate(&self) -> Result<(), Error> {
        if self.device == [0; 16]
            || self.boot == [0; 16]
            || self.session == [0; 16]
            || self.principal == [0; 16]
            || self.diagnostic == 0
            || self.revision == 0
        {
            return Err(Error::Identity);
        }
        if !(1..=600_000).contains(&self.remaining_ms) {
            return Err(Error::Limits);
        }
        Ok(())
    }
    /// # Errors
    /// Invalid data cannot represent an opened installation worker.
    pub fn encode(&self) -> Result<[u8; APPLICATION_RECEIPT_BYTES], Error> {
        self.validate()?;
        let mut out = [0; APPLICATION_RECEIPT_BYTES];
        out[..8].copy_from_slice(HEADER);
        out[8..24].copy_from_slice(&self.device);
        out[24..40].copy_from_slice(&self.boot);
        out[40..48].copy_from_slice(&self.diagnostic.to_le_bytes());
        out[48..64].copy_from_slice(&self.session);
        out[64..80].copy_from_slice(&self.principal);
        out[80..88].copy_from_slice(&self.revision.to_le_bytes());
        out[88..92].copy_from_slice(&1_u32.to_le_bytes()); // installation only
        out[92..94].copy_from_slice(&AUTHENTICATED_APPLICATION.to_le_bytes());
        out[94..96].copy_from_slice(&1_u16.to_le_bytes()); // transfer version
        out[96..98].copy_from_slice(&MESSAGE_BYTES.to_le_bytes());
        out[100..104].copy_from_slice(&6000_u32.to_le_bytes());
        out[104..108].copy_from_slice(&self.remaining_ms.to_le_bytes());
        Ok(out)
    }
    /// # Errors
    /// Reject unknown rights, versions, reserved bytes, budgets and identities.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != APPLICATION_RECEIPT_BYTES
            || &bytes[..8] != HEADER
            || bytes[88..92] != 1_u32.to_le_bytes()
            || bytes[92..94] != AUTHENTICATED_APPLICATION.to_le_bytes()
            || bytes[94..96] != 1_u16.to_le_bytes()
            || bytes[96..98] != MESSAGE_BYTES.to_le_bytes()
            || bytes[98..100] != [0; 2]
            || bytes[100..104] != 6000_u32.to_le_bytes()
            || bytes[108..] != [0; 4]
        {
            return Err(Error::Format);
        }
        let value = Self {
            device: core::array::from_fn(|i| bytes[8 + i]),
            boot: core::array::from_fn(|i| bytes[24 + i]),
            diagnostic: u64::from_le_bytes(core::array::from_fn(|i| bytes[40 + i])),
            session: core::array::from_fn(|i| bytes[48 + i]),
            principal: core::array::from_fn(|i| bytes[64 + i]),
            revision: u64::from_le_bytes(core::array::from_fn(|i| bytes[80 + i])),
            remaining_ms: u32::from_le_bytes(core::array::from_fn(|i| bytes[104 + i])),
        };
        value.validate()?;
        Ok(value)
    }
    /// Caller MUST also authenticate this entire receipt in the current channel.
    /// # Errors
    /// Old or foreign connection, principal and permission revision fail closed.
    pub fn correlate(&self, expected: &Self) -> Result<(), Error> {
        self.validate()?;
        expected.validate()?;
        if self.device == expected.device
            && self.boot == expected.boot
            && self.diagnostic == expected.diagnostic
            && self.session == expected.session
            && self.principal == expected.principal
            && self.revision == expected.revision
            && self.remaining_ms <= expected.remaining_ms
        {
            Ok(())
        } else {
            Err(Error::Identity)
        }
    }
}
