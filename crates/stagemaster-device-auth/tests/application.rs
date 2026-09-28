#![cfg(feature = "application")]
#[path = "../../stagemaster-device-session/tests/support/mod.rs"]
mod support;
use stagemaster_device_auth::application::{DevelopmentPermit, Error, MAX_PERMISSION_MS, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};
use support::{channels, context, key, unconfirmed};

fn permit(duration: u32) -> DevelopmentPermit {
    DevelopmentPermit::installation(context().device, key(3).public(), [9; 16], 7, duration)
        .unwrap()
}

#[test]
fn only_confirmed_expected_controller_can_receive_installation_permission() {
    let (_, server) = unconfirmed();
    assert!(matches!(
        Session::admit(server, permit(10_000), context(), 0),
        Err(Error::Denied)
    ));
    let (_, server) = channels();
    let mut access = Session::admit(server, permit(10_000), context(), 10).unwrap();
    let grant = access.grant(10).unwrap();
    assert_eq!(grant.context(), context());
    assert_eq!(grant.principal(), [9; 16]);
    assert_eq!(grant.revision(), 7);
    assert_eq!(grant.expires_at(), 10_010);
}

#[test]
fn foreign_device_boot_connection_and_holder_are_rejected() {
    for choice in 0..5 {
        let (_, server) = channels();
        let mut expected = context();
        let mut device = context().device;
        let mut holder = key(3).public();
        match choice {
            0 => device = [8; 16],
            1 => holder = key(8).public(),
            2 => expected.device = [8; 16],
            3 => expected.boot = [8; 16],
            _ => expected.connection += 1,
        }
        let permit = DevelopmentPermit::installation(device, holder, [9; 16], 7, 1000).unwrap();
        assert!(matches!(
            Session::admit(server, permit, expected, 0),
            Err(Error::Denied)
        ));
    }
}

#[test]
fn permits_are_bounded_and_have_no_anonymous_or_zero_revision_mode() {
    for choice in 0..6 {
        let mut device = [1; 16];
        let mut holder = key(3).public();
        let mut principal = [9; 16];
        let mut revision = 7;
        let mut duration = 1000;
        match choice {
            0 => device = [0; 16],
            1 => holder = [0; 32],
            2 => principal = [0; 16],
            3 => revision = 0,
            4 => duration = 0,
            _ => duration = MAX_PERMISSION_MS + 1,
        }
        assert!(matches!(
            DevelopmentPermit::installation(device, holder, principal, revision, duration),
            Err(Error::Invalid)
        ));
    }
}

#[test]
fn authenticated_heartbeats_renew_link_but_not_permission() {
    let (mut client, server) = channels();
    let mut access = Session::admit(server, permit(10_000), context(), 0).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let mut plain = [0; PLAINTEXT_BYTES];
    for now in [2000, 4000, 6000, 8000, 9999] {
        let n = client.seal(Kind::Heartbeat, &[], &mut cipher, now).unwrap();
        assert_eq!(
            access.open(&cipher[..n], &mut plain, now).unwrap().kind,
            Kind::Heartbeat
        );
        let n = access
            .seal(Kind::HeartbeatReply, &[], &mut cipher, now)
            .unwrap();
        client.open(&cipher[..n], &mut plain, now).unwrap();
        assert_eq!(access.grant(now).unwrap().expires_at(), 10_000);
    }
    assert_eq!(access.grant(10_000), Err(Error::Expired));
    assert_eq!(access.grant(10_001), Err(Error::Closed));
}

#[test]
fn cached_proof_cannot_restore_revoked_authority_or_cross_connections() {
    let (_, server) = channels();
    let mut access = Session::admit(server, permit(10_000), context(), 0).unwrap();
    let old = access.grant(0).unwrap();
    access.revoke();
    assert_eq!(access.grant(0), Err(Error::Closed));
    let (_, server) = channels();
    let mut new = Session::admit(server, permit(10_000), context(), 0).unwrap();
    assert_ne!(new.grant(0).unwrap().session(), old.session());
}

#[test]
fn idle_security_expiry_clock_rollback_and_tampering_clear_authority() {
    for choice in 0..3 {
        let (mut client, server) = channels();
        let mut access = Session::admit(server, permit(10_000), context(), 100).unwrap();
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let mut plain = [77; PLAINTEXT_BYTES];
        let n = client
            .seal(Kind::Message, b"request", &mut cipher, 100)
            .unwrap();
        let now = match choice {
            0 => 6000,
            1 => 99,
            _ => {
                cipher[0] ^= 1;
                100
            }
        };
        assert!(access.open(&cipher[..n], &mut plain, now).is_err());
        assert_eq!(plain, [0; PLAINTEXT_BYTES]);
        assert_eq!(access.grant(now), Err(Error::Closed));
    }
}

#[test]
fn even_valid_ciphertext_cannot_extend_expired_permission() {
    let (mut client, server) = channels();
    let mut access = Session::admit(server, permit(100), context(), 0).unwrap();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = client.seal(Kind::Heartbeat, &[], &mut cipher, 100).unwrap();
    let mut plain = [1; PLAINTEXT_BYTES];
    assert!(matches!(
        access.open(&cipher[..n], &mut plain, 100),
        Err(Error::Expired)
    ));
    assert_eq!(plain, [0; PLAINTEXT_BYTES]);
}
