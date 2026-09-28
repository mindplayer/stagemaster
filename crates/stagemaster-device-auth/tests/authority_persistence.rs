#![cfg(feature = "persistence")]
mod support;
use stagemaster_device_auth::{
    Vault,
    authority::{Authority, Error, Evidence, Origin, Peer, Security},
    persistence::VaultStore,
};
use support::{Failure, Flash, binding, local, run};

fn evidence(origin: Origin) -> Evidence {
    Evidence {
        security: Security::Authenticated,
        bonded: true,
        origin,
        peer: Peer::from_binding(&binding(1)),
    }
}

#[test]
fn admission_commit_reopen_and_resumed_proof_use_the_actual_durable_binding() {
    let flash = Flash::new();
    let mut store = VaultStore::new(flash.clone(), 0).unwrap();
    let mut auth = Authority::new(run(store.initialize(local())).unwrap().clone(), 0);
    auth.open_pairing(0).unwrap();
    let connection = auth.connect([8; 16], 0).unwrap();
    auth.begin_pairing(connection, 1).unwrap();
    let proposal = auth
        .paired(connection, &evidence(Origin::Pairing), [1; 16], 2)
        .unwrap();
    run(store.commit(&proposal)).unwrap();
    // The committed result cannot authorize the now closed physical connection.
    auth.restore(store.current().unwrap().clone()).unwrap();
    assert_eq!(
        auth.grant(connection, Security::Authenticated, 3),
        Err(Error::Closed)
    );
    drop(store);
    let mut reopened = VaultStore::new(flash, 0).unwrap();
    let verified = run(reopened.recover()).unwrap().clone();
    let mut rebooted = Authority::new(verified, 0);
    let new = rebooted.connect([9; 16], 0).unwrap();
    let grant = rebooted
        .resumed(new, &evidence(Origin::Resumed), 1)
        .unwrap();
    assert_eq!(grant.principal(), [1; 16]);
    assert_eq!(grant.session(), [9; 16]);
}

#[test]
fn uncertain_revoke_never_leaves_old_live_authority_and_recovery_uses_durable_outcome() {
    let initial_flash = Flash::new();
    let mut initial = VaultStore::new(initial_flash.clone(), 0).unwrap();
    let proposal = run(initial.initialize(local()))
        .unwrap()
        .enroll(binding(1))
        .unwrap();
    let confirmed = run(initial.commit(&proposal)).unwrap().clone();
    drop(initial);
    let bytes = initial_flash.0.borrow().bytes.clone();
    let revoked = confirmed.revoke([1; 16]).unwrap();
    let total = {
        let flash = Flash::from(bytes.clone());
        let mut store = VaultStore::new(flash.clone(), 0).unwrap();
        run(store.recover()).unwrap();
        flash.0.borrow_mut().ops = 0;
        run(store.commit(&revoked)).unwrap();
        flash.0.borrow().ops
    };
    for operation in 1..=total {
        for failure in [Failure::Before, Failure::Partial, Failure::After] {
            let flash = Flash::from(bytes.clone());
            let mut store = VaultStore::new(flash.clone(), 0).unwrap();
            let mut auth = Authority::new(run(store.recover()).unwrap().clone(), 0);
            let connection = auth.connect([8; 16], 0).unwrap();
            auth.resumed(connection, &evidence(Origin::Resumed), 0)
                .unwrap();
            let proposal = auth.revoke([1; 16]).unwrap();
            {
                let mut state = flash.0.borrow_mut();
                state.ops = 0;
                state.fail = Some((operation, failure));
            }
            assert!(
                run(store.commit(&proposal)).is_err(),
                "{operation} {failure:?}"
            );
            assert_eq!(
                auth.grant(connection, Security::Authenticated, 1),
                Err(Error::Closed)
            );
            assert_eq!(auth.connect([9; 16], 1), Err(Error::Unavailable));
            assert!(store.current().is_none());
            flash.0.borrow_mut().fail = None;
            drop(store);
            let mut store = VaultStore::new(flash, 0).unwrap();
            if let Ok(verified) = run(store.recover()) {
                assert!(verified == &confirmed || verified == &revoked);
                let expected = verified.find([1; 16]).is_some();
                auth.restore(verified.clone()).unwrap();
                let new = auth.connect([9; 16], 2).unwrap();
                assert_eq!(
                    auth.resumed(new, &evidence(Origin::Resumed), 2).is_ok(),
                    expected
                );
            } else {
                assert!(auth.vault().is_none());
            }
        }
    }
    println!("authority + durable revoke fault points: {total} x 3");
    assert_eq!(confirmed.local(), Vault::new(local()).local());
}
