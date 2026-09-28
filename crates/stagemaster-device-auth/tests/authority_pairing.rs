mod authority_support;
use authority_support::{authority, binding, evidence, local, secret, vault};
use stagemaster_device_auth::{
    Address, Code, LocalIdentity, Vault,
    authority::{Authority, Error, Origin, Peer, Security},
};
const AUTH: Security = Security::Authenticated;

#[test]
fn physical_window_limits_attempts_across_reconnects_and_cannot_be_extended() {
    let mut auth = authority();
    let first = auth.connect([1; 16], 0).unwrap();
    assert_eq!(auth.begin_pairing(first, 1), Err(Error::PairingClosed));
    auth.open_pairing(2).unwrap();
    for i in 2..=4 {
        let connection = auth.connect([i; 16], u64::from(i)).unwrap();
        auth.begin_pairing(connection, u64::from(i)).unwrap();
        auth.disconnect(connection);
        assert_eq!(auth.open_pairing(u64::from(i)), Err(Error::Busy));
    }
    let connection = auth.connect([5; 16], 5).unwrap();
    assert_eq!(auth.begin_pairing(connection, 6), Err(Error::Attempts));
    assert_eq!(auth.open_pairing(90001), Err(Error::Busy));
    auth.open_pairing(90002).unwrap();
    let next = auth.connect([6; 16], 90003).unwrap();
    auth.begin_pairing(next, 90003).unwrap();
}

#[test]
fn cancellation_and_deadline_reject_late_pairing_results() {
    for cancelled in [false, true] {
        let mut auth = authority();
        auth.open_pairing(0).unwrap();
        let connection = auth.connect([3; 16], 89998).unwrap();
        auth.begin_pairing(connection, 89999).unwrap();
        if cancelled {
            assert_eq!(auth.close_pairing(), Some(connection));
        }
        assert_eq!(
            auth.paired(connection, &evidence(2, Origin::Pairing), [2; 16], 90000),
            Err(if cancelled {
                Error::Closed
            } else {
                Error::Expired
            })
        );
        assert!(auth.vault().unwrap().find([2; 16]).is_none());
    }
}

#[test]
fn pairing_yields_only_a_proposal_and_requires_postcommit_reconnection() {
    let mut auth = authority();
    auth.open_pairing(0).unwrap();
    let connection = auth.connect([3; 16], 1).unwrap();
    auth.begin_pairing(connection, 2).unwrap();
    let proposal = auth
        .paired(connection, &evidence(2, Origin::Pairing), [2; 16], 3)
        .unwrap();
    assert!(auth.vault().is_none());
    assert_eq!(auth.grant(connection, AUTH, 4), Err(Error::Closed));
    assert_eq!(auth.connect([4; 16], 4), Err(Error::Unavailable));
    // Core-only fixture: persistence-backed roundtrip is tested separately.
    auth.restore(proposal).unwrap();
    let next = auth.connect([4; 16], 5).unwrap();
    assert_eq!(
        auth.paired(connection, &evidence(2, Origin::Pairing), [2; 16], 5),
        Err(Error::Stale)
    );
    assert_eq!(auth.grant(next, AUTH, 5), Err(Error::Denied));
    assert_eq!(
        auth.resumed(next, &evidence(2, Origin::Resumed), 6)
            .unwrap()
            .principal(),
        [2; 16]
    );
}

#[test]
fn pairing_cannot_use_a_weak_proof_or_a_resumption_event() {
    for (security, bonded, origin) in [
        (Security::Encrypted, true, Origin::Pairing),
        (AUTH, false, Origin::Pairing),
        (AUTH, true, Origin::Resumed),
    ] {
        let mut auth = authority();
        auth.open_pairing(0).unwrap();
        let connection = auth.connect([3; 16], 0).unwrap();
        auth.begin_pairing(connection, 0).unwrap();
        let mut proof = evidence(2, origin);
        proof.security = security;
        proof.bonded = bonded;
        assert_eq!(
            auth.paired(connection, &proof, [2; 16], 1),
            Err(Error::Denied)
        );
        assert_eq!(auth.vault(), Some(&vault()));
        assert_eq!(auth.grant(connection, AUTH, 2), Err(Error::Closed));
    }
}

#[test]
fn key_refresh_keeps_stored_principal_and_invalidates_old_key() {
    let mut auth = authority();
    auth.open_pairing(0).unwrap();
    let connection = auth.connect([3; 16], 0).unwrap();
    auth.begin_pairing(connection, 0).unwrap();
    let mut proof = evidence(1, Origin::Pairing);
    proof.peer = Peer::new(binding(1).address(), secret(99), binding(1).irk().cloned());
    let proposal = auth.paired(connection, &proof, [42; 16], 1).unwrap();
    assert!(proposal.find([42; 16]).is_none());
    assert_eq!(proposal.find([1; 16]).unwrap().ltk(), &secret(99));
    auth.restore(proposal).unwrap();
    let next = auth.connect([4; 16], 2).unwrap();
    assert_eq!(
        auth.resumed(next, &evidence(1, Origin::Resumed), 2),
        Err(Error::Denied)
    );
    let next = auth.connect([5; 16], 3).unwrap();
    proof.origin = Origin::Resumed;
    assert_eq!(auth.resumed(next, &proof, 3).unwrap().principal(), [1; 16]);
}

#[test]
fn revocation_closes_before_commit_and_recovery_never_revives_old_links() {
    let mut auth = authority();
    let connection = auth.connect([3; 16], 0).unwrap();
    auth.resumed(connection, &evidence(1, Origin::Resumed), 0)
        .unwrap();
    let proposal = auth.revoke([1; 16]).unwrap();
    assert_eq!(auth.grant(connection, AUTH, 1), Err(Error::Closed));
    auth.restore(proposal).unwrap();
    let next = auth.connect([4; 16], 2).unwrap();
    assert_eq!(
        auth.resumed(next, &evidence(1, Origin::Resumed), 2),
        Err(Error::Denied)
    );
    auth.suspend();
    assert_eq!(auth.restore(vault()), Err(Error::Binding(Code::Conflict)));
    assert!(auth.vault().is_none());
}

#[test]
fn recovery_refuses_another_local_identity_and_keeps_available_state_on_busy() {
    let mut auth = authority();
    assert_eq!(auth.restore(vault()), Err(Error::Busy));
    assert_eq!(auth.vault(), Some(&vault()));
    auth.suspend();
    let other = LocalIdentity::new(
        Address::new(true, [3, 4, 5, 6, 7, 0xc8]).unwrap(),
        secret(99),
    )
    .unwrap();
    assert_eq!(
        auth.restore(Vault::new(other)),
        Err(Error::Binding(Code::Conflict))
    );
    auth.restore(vault()).unwrap();
}

#[test]
fn full_binding_table_cannot_silently_evict_an_existing_owner() {
    let mut full = Vault::new(local());
    for i in 1..=4 {
        full = full.enroll(binding(i)).unwrap();
    }
    let mut auth = Authority::new(full.clone(), 0);
    auth.open_pairing(0).unwrap();
    let connection = auth.connect([6; 16], 0).unwrap();
    auth.begin_pairing(connection, 0).unwrap();
    assert_eq!(
        auth.paired(connection, &evidence(5, Origin::Pairing), [5; 16], 1),
        Err(Error::Binding(Code::Full))
    );
    assert_eq!(auth.vault(), Some(&full));
    assert_eq!(auth.grant(connection, AUTH, 2), Err(Error::Closed));
}
