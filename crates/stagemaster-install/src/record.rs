use crate::{Code, Identity, Slot};
use sha2::{Digest, Sha256};
pub const RECORD_BYTES: usize = 96;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Commit {
    pub slot: Slot,
    pub generation: u64,
    pub identity: Identity,
}
impl Commit {
    /// # Errors
    /// Reject zero generations and invalid package sizes before producing metadata.
    pub fn encode(self) -> Result<[u8; RECORD_BYTES], Code> {
        self.identity.validate()?;
        if self.generation == 0 {
            return Err(Code::Metadata);
        }
        let mut b = [0; RECORD_BYTES];
        b[..8].copy_from_slice(b"STMINST\0");
        b[8] = 1;
        b[10] = u8::from(self.slot == Slot::B);
        b[16..24].copy_from_slice(&self.generation.to_le_bytes());
        b[24..28].copy_from_slice(
            &u32::try_from(self.identity.bytes)
                .map_err(|_| Code::Bounds)?
                .to_le_bytes(),
        );
        b[32..64].copy_from_slice(&self.identity.digest);
        let hash = Sha256::digest(&b[..64]);
        b[64..].copy_from_slice(&hash);
        Ok(b)
    }
    /// # Errors
    /// Reject unknown versions, reserved fields, wrong slots and damaged metadata.
    pub fn decode(slot: Slot, b: &[u8; RECORD_BYTES]) -> Result<Self, Code> {
        if &b[..8] != b"STMINST\0"
            || b[8..10] != [1, 0]
            || b[10] != u8::from(slot == Slot::B)
            || b[11..16] != [0; 5]
            || b[28..32] != [0; 4]
            || Sha256::digest(&b[..64])[..] != b[64..]
        {
            return Err(Code::Metadata);
        }
        let generation = u64::from_le_bytes(core::array::from_fn(|i| b[16 + i]));
        if generation == 0 {
            return Err(Code::Metadata);
        }
        let identity = Identity {
            bytes: usize::try_from(u32::from_le_bytes(core::array::from_fn(|i| b[24 + i])))
                .map_err(|_| Code::Bounds)?,
            digest: core::array::from_fn(|i| b[32 + i]),
        };
        identity.validate()?;
        Ok(Self {
            slot,
            generation,
            identity,
        })
    }
}
