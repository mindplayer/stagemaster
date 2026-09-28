use crate::Code;
use zeroize::Zeroizing;

/// Never print keys; zeroize the owned bytes when this value is dropped.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(Zeroizing<[u8; 16]>);
impl core::fmt::Debug for Secret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl Secret {
    /// # Errors
    /// A zero key is not an initialized credential.
    pub fn new(bytes: [u8; 16]) -> Result<Self, Code> {
        let bytes = Zeroizing::new(bytes);
        if *bytes == [0; 16] {
            return Err(Code::Invalid);
        }
        Ok(Self(bytes))
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    pub(crate) random: bool,
    pub(crate) bytes: [u8; 6],
}
impl Address {
    /// Identity addresses only, not a connection's transient private address.
    /// # Errors
    /// Refuses uninitialized addresses and non-static random identity addresses.
    pub fn new(random: bool, bytes: [u8; 6]) -> Result<Self, Code> {
        let lower = [
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5] & 0x3f,
        ];
        if bytes == [0; 6]
            || bytes == [0xff; 6]
            || (random
                && (bytes[5] & 0xc0 != 0xc0
                    || lower == [0; 6]
                    || lower == [0xff, 0xff, 0xff, 0xff, 0xff, 0x3f]))
        {
            return Err(Code::Invalid);
        }
        Ok(Self { random, bytes })
    }
    #[must_use]
    pub const fn bytes(self) -> [u8; 6] {
        self.bytes
    }
    #[must_use]
    pub const fn is_random(self) -> bool {
        self.random
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalIdentity {
    pub(crate) address: Address,
    pub(crate) irk: Secret,
}
impl LocalIdentity {
    /// # Errors
    /// The current format requires a persisted random-static local identity.
    pub fn new(address: Address, irk: Secret) -> Result<Self, Code> {
        if !address.random {
            return Err(Code::Invalid);
        }
        Ok(Self { address, irk })
    }
    #[must_use]
    pub const fn address(&self) -> Address {
        self.address
    }
    #[must_use]
    pub const fn irk(&self) -> &Secret {
        &self.irk
    }
}

/// Construct only from a trusted LESC authenticated, bonded pairing result.
/// This record alone does not prove key possession on any live connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub(crate) principal: [u8; 16],
    pub(crate) address: Address,
    pub(crate) ltk: Secret,
    pub(crate) irk: Option<Secret>,
}
impl Binding {
    /// # Errors
    /// The host must supply a nonzero random, stable local principal identifier.
    pub fn new(
        principal: [u8; 16],
        address: Address,
        ltk: Secret,
        irk: Option<Secret>,
    ) -> Result<Self, Code> {
        if principal == [0; 16] {
            return Err(Code::Invalid);
        }
        Ok(Self {
            principal,
            address,
            ltk,
            irk,
        })
    }
    #[must_use]
    pub const fn principal(&self) -> [u8; 16] {
        self.principal
    }
    #[must_use]
    pub const fn address(&self) -> Address {
        self.address
    }
    #[must_use]
    pub const fn ltk(&self) -> &Secret {
        &self.ltk
    }
    #[must_use]
    pub const fn irk(&self) -> Option<&Secret> {
        self.irk.as_ref()
    }
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        self.address == other.address
            || self
                .irk
                .as_ref()
                .zip(other.irk.as_ref())
                .is_some_and(|(a, b)| a == b)
    }
}
