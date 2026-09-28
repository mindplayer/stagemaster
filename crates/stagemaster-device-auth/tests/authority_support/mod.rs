use stagemaster_device_auth::{
    Address, Binding, LocalIdentity, Secret, Vault,
    authority::{Authority, Evidence, Origin, Peer, Security},
};

pub fn secret(value: u8) -> Secret {
    Secret::new([value; 16]).unwrap()
}
pub fn local() -> LocalIdentity {
    LocalIdentity::new(
        Address::new(true, [1, 2, 3, 4, 5, 0xc6]).unwrap(),
        secret(50),
    )
    .unwrap()
}
pub fn binding(value: u8) -> Binding {
    Binding::new(
        [value; 16],
        Address::new(false, [value; 6]).unwrap(),
        secret(value),
        Some(secret(value + 20)),
    )
    .unwrap()
}
pub fn evidence(value: u8, origin: Origin) -> Evidence {
    Evidence {
        security: Security::Authenticated,
        bonded: true,
        origin,
        peer: Peer::from_binding(&binding(value)),
    }
}
pub fn vault() -> Vault {
    Vault::new(local()).enroll(binding(1)).unwrap()
}
pub fn authority() -> Authority {
    Authority::new(vault(), 0)
}
