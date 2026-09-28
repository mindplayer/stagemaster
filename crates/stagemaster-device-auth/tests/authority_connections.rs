mod authority_support;
use authority_support::{authority, binding, evidence, secret};
use stagemaster_device_auth::authority::{Error, Origin, Peer, Security};
const AUTH: Security = Security::Authenticated;

#[test]
fn known_authenticated_key_restoration_selects_stored_principal() {
    let mut auth = authority();
    let connection = auth.connect([3; 16], 10).unwrap();
    assert_eq!(auth.grant(connection, AUTH, 11), Err(Error::Denied));
    let grant = auth
        .resumed(connection, &evidence(1, Origin::Resumed), 12)
        .unwrap();
    assert_eq!(grant.principal(), binding(1).principal());
    assert_eq!(grant.session(), [3; 16]);
    assert_eq!(grant.connection(), connection);
    assert_eq!(connection.epoch().get(), 1);
    assert_eq!(auth.grant(connection, AUTH, 6011).unwrap(), grant);
    assert_eq!(auth.grant(connection, AUTH, 6012), Err(Error::Expired));
    assert_eq!(
        auth.resumed(connection, &evidence(1, Origin::Resumed), 6012),
        Err(Error::Closed)
    );
}

#[test]
fn weak_or_mismatched_key_evidence_cannot_be_retried_into_authority() {
    let mut cases = Vec::new();
    for security in [Security::Unencrypted, Security::Encrypted] {
        let mut proof = evidence(1, Origin::Resumed);
        proof.security = security;
        cases.push(proof);
    }
    let mut unbonded = evidence(1, Origin::Resumed);
    unbonded.bonded = false;
    cases.push(unbonded);
    cases.push(evidence(1, Origin::Pairing));
    cases.push(evidence(2, Origin::Resumed));
    for peer in [
        Peer::new(binding(1).address(), secret(2), binding(1).irk().cloned()),
        Peer::new(binding(2).address(), secret(1), binding(1).irk().cloned()),
        Peer::new(binding(1).address(), secret(1), None),
        Peer::new(binding(1).address(), secret(1), Some(secret(90))),
    ] {
        let mut proof = evidence(1, Origin::Resumed);
        proof.peer = peer;
        cases.push(proof);
    }
    for proof in cases {
        let mut auth = authority();
        let connection = auth.connect([3; 16], 0).unwrap();
        assert_eq!(auth.resumed(connection, &proof, 1), Err(Error::Denied));
        assert_eq!(
            auth.resumed(connection, &evidence(1, Origin::Resumed), 2),
            Err(Error::Closed)
        );
    }
}

#[test]
fn only_valid_heartbeat_renews_and_downgrade_closes() {
    let mut auth = authority();
    let connection = auth.connect([3; 16], 0).unwrap();
    auth.resumed(connection, &evidence(1, Origin::Resumed), 0)
        .unwrap();
    auth.heartbeat(connection, AUTH, 5999).unwrap();
    assert!(auth.grant(connection, AUTH, 11998).is_ok());
    assert_eq!(auth.heartbeat(connection, AUTH, 11999), Err(Error::Expired));
    let next = auth.connect([4; 16], 12000).unwrap();
    auth.resumed(next, &evidence(1, Origin::Resumed), 12000)
        .unwrap();
    assert_eq!(
        auth.grant(next, Security::Encrypted, 12001),
        Err(Error::Denied)
    );
    assert_eq!(auth.grant(next, AUTH, 12002), Err(Error::Closed));
}

#[test]
fn old_events_and_backwards_stale_timestamps_do_not_touch_new_connection() {
    let mut auth = authority();
    assert_eq!(auth.connect([0; 16], 0), Err(Error::Invalid));
    let old = auth.connect([3; 16], 0).unwrap();
    assert_eq!(auth.connect([4; 16], 1), Err(Error::Busy));
    auth.disconnect(old);
    assert_eq!(auth.connect([3; 16], 10), Err(Error::Invalid));
    let new = auth.connect([4; 16], 10).unwrap();
    assert!(new.epoch() > old.epoch());
    auth.resumed(new, &evidence(1, Origin::Resumed), 10)
        .unwrap();
    assert_eq!(
        auth.resumed(old, &evidence(1, Origin::Resumed), 0),
        Err(Error::Stale)
    );
    assert_eq!(auth.heartbeat(old, AUTH, 0), Err(Error::Stale));
    auth.disconnect(old);
    assert!(auth.grant(new, AUTH, 11).is_ok());
}

#[test]
fn clock_failure_closes_authority_and_window_without_wraparound() {
    let mut auth = authority();
    auth.open_pairing(10).unwrap();
    let connection = auth.connect([3; 16], 10).unwrap();
    auth.resumed(connection, &evidence(1, Origin::Resumed), 10)
        .unwrap();
    assert_eq!(auth.grant(connection, AUTH, 9), Err(Error::Clock));
    assert_eq!(auth.grant(connection, AUTH, 10), Err(Error::Closed));
    let next = auth.connect([4; 16], 11).unwrap();
    assert_eq!(auth.begin_pairing(next, 12), Err(Error::PairingClosed));
    assert_eq!(auth.connect([5; 16], u64::MAX), Err(Error::Exhausted));
    assert_eq!(auth.open_pairing(u64::MAX), Err(Error::Exhausted));
}

#[test]
fn idle_admission_has_a_fixed_deadline_and_pairing_failure_has_no_fallback() {
    let mut auth = authority();
    let connection = auth.connect([3; 16], 1).unwrap();
    assert_eq!(auth.grant(connection, AUTH, 89999), Err(Error::Denied));
    assert_eq!(auth.poll(90001).unwrap(), Some(connection));
    assert_eq!(
        auth.resumed(connection, &evidence(1, Origin::Resumed), 90001),
        Err(Error::Closed)
    );
}
