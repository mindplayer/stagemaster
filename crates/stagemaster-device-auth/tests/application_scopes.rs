#![cfg(feature = "application")]
#[path = "../../stagemaster-device-session/tests/support/mod.rs"]
mod support;
use stagemaster_device_auth::application::{
    Configuration, DevelopmentPermit, Error, Permissions, Role, Scope, Session,
};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Channel, Kind, PLAINTEXT_BYTES};
use support::{channels, context, key};

const SCOPES: [Scope; 3] = [Scope::Installation, Scope::Observe, Scope::Control];

fn pair(permissions: Permissions, duration: u32) -> (Channel, Session) {
    let (client, server) = channels();
    let permit = DevelopmentPermit::scoped(
        context().device,
        key(3).public(),
        [9; 16],
        7,
        duration,
        permissions,
    )
    .unwrap();
    (
        client,
        Session::admit(server, permit, context(), 0).unwrap(),
    )
}

#[test]
fn operation_scopes_are_independent_and_combinations_are_explicit() {
    for scopes in [
        vec![Scope::Installation],
        vec![Scope::Observe],
        vec![Scope::Control],
        vec![Scope::Installation, Scope::Observe],
        vec![Scope::Installation, Scope::Control],
        vec![Scope::Observe, Scope::Control],
        SCOPES.to_vec(),
    ] {
        let permissions = scopes
            .iter()
            .fold(Permissions::only(scopes[0]), |p, scope| p.with(*scope));
        for requested in SCOPES {
            let (_, mut access) = pair(permissions, 1000);
            let result = access.require(requested, 0);
            if scopes.contains(&requested) {
                assert_eq!(result.unwrap().permissions(), permissions);
            } else {
                assert_eq!(result, Err(Error::Denied));
                assert_eq!(access.require(scopes[0], 0), Err(Error::Closed));
            }
        }
    }
}

#[test]
fn successful_identity_and_heartbeats_cannot_add_a_missing_scope() {
    let (mut client, mut access) = pair(Permissions::only(Scope::Observe), 1000);
    let old = access.grant(0).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let mut plain = [1; PLAINTEXT_BYTES];
    let n = client.seal(Kind::Heartbeat, &[], &mut cipher, 10).unwrap();
    access.open(&cipher[..n], &mut plain, 10).unwrap();
    assert_eq!(access.require(Scope::Control, 10), Err(Error::Denied));
    assert!(old.permissions().contains(Scope::Observe));
    let n = client.seal(Kind::Heartbeat, &[], &mut cipher, 11).unwrap();
    assert!(matches!(
        access.open(&cipher[..n], &mut plain, 11),
        Err(Error::Closed)
    ));
    assert_eq!(plain, [0; PLAINTEXT_BYTES]);
}

#[test]
fn operation_checks_revalidate_expiry_and_do_not_inherit_prior_connection_scopes() {
    let all = Permissions::only(Scope::Observe)
        .with(Scope::Control)
        .with(Scope::Installation);
    let (mut client, mut access) = pair(all, 100);
    let old = access.require(Scope::Control, 0).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let mut plain = [0; PLAINTEXT_BYTES];
    let n = client.seal(Kind::Heartbeat, &[], &mut cipher, 99).unwrap();
    access.open(&cipher[..n], &mut plain, 99).unwrap();
    assert!(access.require(Scope::Control, 99).is_ok());
    assert_eq!(access.require(Scope::Control, 100), Err(Error::Expired));
    assert_eq!(access.require(Scope::Observe, 100), Err(Error::Closed));
    let (_, mut new) = pair(Permissions::only(Scope::Observe), 1000);
    assert_ne!(
        new.require(Scope::Observe, 0).unwrap().session(),
        old.session()
    );
    assert_eq!(new.require(Scope::Control, 0), Err(Error::Denied));
}

#[test]
fn existing_local_v1_configuration_only_grants_installation() {
    let mut bytes = [0; 160];
    bytes[..8].copy_from_slice(b"SMDV\x01\x01\xa0\0");
    bytes[8..24].copy_from_slice(&context().device);
    bytes[24..40].fill(9);
    bytes[40..48].copy_from_slice(&7_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&1000_u32.to_le_bytes());
    bytes[56..88].fill(4);
    bytes[88..120].copy_from_slice(&key(3).public());
    bytes[120..152].copy_from_slice(&key(4).public());
    let config = Configuration::import(&bytes, Role::Device).unwrap();
    for scope in SCOPES {
        let (_, server) = channels();
        let mut access = Session::admit(server, config.permit().unwrap(), context(), 0).unwrap();
        let grant = access.grant(0).unwrap();
        assert_eq!(grant.permissions(), Permissions::only(Scope::Installation));
        assert_eq!(
            access.require(scope, 0).is_ok(),
            scope == Scope::Installation
        );
    }
}
