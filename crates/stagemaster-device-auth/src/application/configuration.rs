//! Explicit, local development provisioning. Never accept this format over GATT.
use super::{DevelopmentPermit, Error, Permissions, Scope};
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
    permissions: Permissions,
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
            || &bytes[..4] != b"SMDV"
            || !matches!(bytes[4], 1 | 2)
            || bytes[6..8] != [160, 0]
            || bytes[53..56] != [0; 3]
            || bytes[152..] != [0; 8]
        {
            return Err(Error::Invalid);
        }
        let permissions = match bytes[4] {
            1 if bytes[52] == 0 => Permissions::only(Scope::Installation),
            2 if (1..=7).contains(&bytes[52]) => {
                let bits = bytes[52];
                let mut result = if bits & 1 != 0 {
                    Permissions::only(Scope::Installation)
                } else if bits & 2 != 0 {
                    Permissions::only(Scope::Observe)
                } else {
                    Permissions::only(Scope::Control)
                };
                for scope in [Scope::Installation, Scope::Observe, Scope::Control] {
                    if bits & scope as u8 != 0 {
                        result = result.with(scope);
                    }
                }
                result
            }
            _ => return Err(Error::Invalid),
        };
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
            permissions,
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
    #[must_use]
    pub const fn permissions(&self) -> Permissions {
        self.permissions
    }
    /// Explicit local v2 runtime policy. Legacy files never acquire runtime access.
    /// # Errors
    /// Only a device-side configuration with observation scope can admit this endpoint.
    pub fn runtime_permit(&self) -> Result<DevelopmentPermit, Error> {
        if self.role != Role::Device || !self.permissions.contains(Scope::Observe) {
            return Err(Error::Denied);
        }
        DevelopmentPermit::scoped(
            self.device,
            self.trusted,
            self.principal,
            self.revision,
            self.duration_ms,
            self.permissions,
        )
    }
    /// # Errors
    /// A controller-side file cannot grant server-side installation rights.
    pub fn permit(&self) -> Result<DevelopmentPermit, Error> {
        if self.role != Role::Device || !self.permissions.contains(Scope::Installation) {
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
