//! Stack-owned authenticated evidence only. Never accepts bytes from GATT.
use stagemaster_device_auth::authority::{Evidence, Origin, Peer, Security};
use stagemaster_device_auth::{Address as StoredAddress, Binding, Secret};
use trouble_host::prelude::*;

pub fn evidence(bond: &BondInformation, origin: Origin, level: SecurityLevel) -> Option<Evidence> {
    if level != SecurityLevel::EncryptedAuthenticated
        || bond.security_level != level
        || !bond.is_bonded
    {
        return None;
    }
    let address = StoredAddress::new(
        bond.identity.addr.kind.into_inner() & 1 == 1,
        bond.identity.addr.addr.into_inner(),
    )
    .ok()?;
    let ltk = Secret::new(u128::from(&bond.ltk).to_le_bytes()).ok()?;
    let irk = match bond.identity.irk.as_ref() {
        Some(key) => Some(Secret::new(u128::from(key).to_le_bytes()).ok()?),
        None => None,
    };
    Some(Evidence {
        security: Security::Authenticated,
        bonded: true,
        origin,
        peer: Peer::new(address, ltk, irk),
    })
}

pub fn bond(binding: &Binding) -> BondInformation {
    let address = binding.address();
    BondInformation::new(
        Identity {
            addr: Address::new(
                if address.is_random() {
                    AddrKind::RANDOM
                } else {
                    AddrKind::PUBLIC
                },
                BdAddr::new(address.bytes()),
            ),
            irk: binding
                .irk()
                .and_then(|key| IdentityResolvingKey::new(u128::from_le_bytes(*key.bytes()))),
        },
        LongTermKey::new(u128::from_le_bytes(*binding.ltk().bytes())),
        SecurityLevel::EncryptedAuthenticated,
        true,
    )
}

pub fn security(level: Result<SecurityLevel, trouble_host::Error>) -> Security {
    match level {
        Ok(SecurityLevel::EncryptedAuthenticated) => Security::Authenticated,
        Ok(SecurityLevel::Encrypted) => Security::Encrypted,
        _ => Security::Unencrypted,
    }
}
