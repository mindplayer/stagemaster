use crate::{Address, Binding, Code, LocalIdentity, MAX_BINDINGS, Secret, Vault};
use zeroize::Zeroizing;
pub const RECORD_BYTES: usize = 384;
pub struct SecretRecord(Zeroizing<[u8; RECORD_BYTES]>);
impl core::fmt::Debug for SecretRecord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl core::ops::Deref for SecretRecord {
    type Target = [u8; RECORD_BYTES];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
const MAGIC: &[u8; 8] = b"SMBOND01";

impl Vault {
    #[must_use]
    pub fn encode(&self) -> SecretRecord {
        let mut out = Zeroizing::new([0; RECORD_BYTES]);
        out[..8].copy_from_slice(MAGIC);
        out[8..16].copy_from_slice(&self.generation.to_le_bytes());
        out[16..22].copy_from_slice(&self.local.address.bytes);
        out[24..40].copy_from_slice(self.local.irk.bytes());
        for (index, binding) in self.bindings().enumerate() {
            out[40] += 1;
            let entry = &mut out[64 + index * 80..][..80];
            entry[..16].copy_from_slice(&binding.principal);
            entry[16] = u8::from(binding.address.random);
            entry[17..23].copy_from_slice(&binding.address.bytes);
            entry[24..40].copy_from_slice(binding.ltk.bytes());
            if let Some(irk) = &binding.irk {
                entry[23] = 1;
                entry[40..56].copy_from_slice(irk.bytes());
            }
        }
        SecretRecord(out)
    }
    /// # Errors
    /// Strict length, version, reserved bits, identities and duplicate checks. No migration guesses.
    pub fn decode(bytes: &[u8]) -> Result<Self, Code> {
        if bytes.len() != RECORD_BYTES
            || &bytes[..8] != MAGIC
            || bytes[22..24] != [0; 2]
            || bytes[41..64] != [0; 23]
            || usize::from(bytes[40]) > MAX_BINDINGS
        {
            return Err(Code::Format);
        }
        let generation = u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| Code::Format)?);
        if generation == 0 {
            return Err(Code::Format);
        }
        let local = LocalIdentity::new(
            Address::new(true, bytes[16..22].try_into().map_err(|_| Code::Format)?)?,
            Secret::new(bytes[24..40].try_into().map_err(|_| Code::Format)?)?,
        )?;
        let mut result = Self::new(local);
        result.generation = generation;
        for index in 0..MAX_BINDINGS {
            let entry = &bytes[64 + index * 80..][..80];
            if index >= usize::from(bytes[40]) {
                if entry != [0; 80] {
                    return Err(Code::Format);
                }
                continue;
            }
            if entry[16] > 1 || entry[23] > 1 || entry[56..80] != [0; 24] {
                return Err(Code::Format);
            }
            let irk = if entry[23] == 1 {
                Some(Secret::new(
                    entry[40..56].try_into().map_err(|_| Code::Format)?,
                )?)
            } else {
                if entry[40..56] != [0; 16] {
                    return Err(Code::Format);
                }
                None
            };
            let binding = Binding::new(
                entry[..16].try_into().map_err(|_| Code::Format)?,
                Address::new(
                    entry[16] == 1,
                    entry[17..23].try_into().map_err(|_| Code::Format)?,
                )?,
                Secret::new(entry[24..40].try_into().map_err(|_| Code::Format)?)?,
                irk,
            )?;
            if result
                .bindings()
                .any(|b| b.principal == binding.principal || b.same_identity(&binding))
            {
                return Err(Code::Duplicate);
            }
            result.bindings[index] = Some(binding);
        }
        Ok(result)
    }
}
