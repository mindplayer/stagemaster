//! Explicit, local development provisioning. Never accept this format over GATT.
use super::{DevelopmentPermit, Error};
use stagemaster_device_session::SecretKey;
use zeroize::Zeroizing;

pub const CONFIGURATION_BYTES: usize = 160;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Device,
    Controller,
}

/// Intentionally no Debug, Clone, serialization or secret export.
pub struct Configuration {
    role: Role,
    device: [u8; 16],
    principal: [u8; 16],
    revision: u64,
    duration_ms: u32,
    key: SecretKey,
    trusted: [u8; 32],
}
impl Configuration {
    /// Reads only a trusted local file/build input. The caller owns and must clear
    /// the source buffer; parsing alone is NOT an enrollment or authorization flow.
    /// # Errors
    /// Reject malformed formats, role confusion, invalid policy and key mismatches.
    pub fn import(bytes: &[u8], expected_role: Role) -> Result<Self, Error> {
        if bytes.len() != CONFIGURATION_BYTES
            || &bytes[..5] != b"SMDV\x01"
            || bytes[6..8] != [160, 0]
            || bytes[52..56] != [0; 4]
            || bytes[152..] != [0; 8]
        {
            return Err(Error::Invalid);
        }
        let role = match bytes[5] {
            1 => Role::Device,
            2 => Role::Controller,
            _ => return Err(Error::Invalid),
        };
        if role != expected_role {
            return Err(Error::Invalid);
        }
        let secret = Zeroizing::new(core::array::from_fn(|i| bytes[56 + i]));
        let key = SecretKey::import(*secret)?;
        if key.public() != bytes[120..152] {
            return Err(Error::Invalid);
        }
        let value = Self {
            role,
            device: core::array::from_fn(|i| bytes[8 + i]),
            principal: core::array::from_fn(|i| bytes[24 + i]),
            revision: u64::from_le_bytes(core::array::from_fn(|i| bytes[40 + i])),
            duration_ms: u32::from_le_bytes(core::array::from_fn(|i| bytes[48 + i])),
            key,
            trusted: core::array::from_fn(|i| bytes[88 + i]),
        };
        DevelopmentPermit::installation(
            value.device,
            value.trusted,
            value.principal,
            value.revision,
            value.duration_ms,
        )?;
        if value.trusted == value.key.public() {
            return Err(Error::Invalid);
        }
        Ok(value)
    }
    #[must_use]
    pub const fn role(&self) -> Role {
        self.role
    }
    #[must_use]
    pub const fn device(&self) -> [u8; 16] {
        self.device
    }
    #[must_use]
    pub const fn principal(&self) -> [u8; 16] {
        self.principal
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub const fn duration_ms(&self) -> u32 {
        self.duration_ms
    }
    #[must_use]
    pub const fn key(&self) -> &SecretKey {
        &self.key
    }
    #[must_use]
    pub const fn trusted_key(&self) -> [u8; 32] {
        self.trusted
    }
    /// # Errors
    /// A controller-side file cannot grant server-side installation rights.
    pub fn permit(&self) -> Result<DevelopmentPermit, Error> {
        if self.role != Role::Device {
            return Err(Error::Denied);
        }
        DevelopmentPermit::installation(
            self.device,
            self.trusted,
            self.principal,
            self.revision,
            self.duration_ms,
        )
    }
}
