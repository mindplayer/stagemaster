#![cfg(feature = "persistence")]
mod support;
use stagemaster_device_auth::{
    Code,
    persistence::{Error, IoError, PAGE_BYTES, VaultStore},
};
use support::{Failure, Flash, binding, local, run};

#[test]
fn initialize_enroll_reopen_revoke_and_reject_stale_proposals() {
    let flash = Flash::new();
    let mut store = VaultStore::new(flash.clone(), 17).unwrap();
    assert!(run(store.recover()).is_err());
    assert_eq!(flash.0.borrow().writes, 0);
    let initial = run(store.initialize(local())).unwrap().clone();
    let proposed = initial.enroll(binding(1)).unwrap();
    run(store.commit(&proposed)).unwrap();
    assert!(matches!(
        run(store.commit(&proposed)),
        Err(Error::Code(Code::Conflict))
    ));
    assert_eq!(store.current(), Some(&proposed));
    drop(store);
    let mut store = VaultStore::new(flash.clone(), 9).unwrap();
    let writes = flash.0.borrow().writes;
    assert_eq!(run(store.recover()).unwrap(), &proposed);
    assert_eq!(flash.0.borrow().writes, writes);
    let revoked = proposed.revoke([1; 16]).unwrap();
    run(store.commit(&revoked)).unwrap();
    drop(store);
    let mut store = VaultStore::new(flash, 0).unwrap();
    assert_eq!(run(store.recover()).unwrap(), &revoked);
    assert!(run(store.initialize(local())).is_err());
    assert!(store.current().is_none());
}

fn bound_image() -> Vec<u8> {
    let flash = Flash::new();
    let mut store = VaultStore::new(flash.clone(), 0).unwrap();
    let initial = run(store.initialize(local())).unwrap();
    let proposal = initial.enroll(binding(1)).unwrap();
    run(store.commit(&proposal)).unwrap();
    flash.0.borrow().bytes.clone()
}

#[test]
fn every_storage_failure_requires_recovery_and_never_acknowledges_uncertain_revoke() {
    let original = bound_image();
    let flash = Flash::from(original.clone());
    let mut store = VaultStore::new(flash.clone(), 7).unwrap();
    let before = run(store.recover()).unwrap().clone();
    let after = before.revoke([1; 16]).unwrap();
    flash.0.borrow_mut().ops = 0;
    run(store.commit(&after)).unwrap();
    let operations = flash.0.borrow().ops;
    for at in 1..=operations {
        for failure in [Failure::Before, Failure::Partial, Failure::After] {
            let flash = Flash::from(original.clone());
            let mut store = VaultStore::new(flash.clone(), 7).unwrap();
            run(store.recover()).unwrap();
            {
                let mut s = flash.0.borrow_mut();
                s.ops = 0;
                s.fail = Some((at, failure));
            }
            let ok = run(store.commit(&after)).is_ok();
            if !ok {
                assert!(store.current().is_none(), "op {at} {failure:?}");
            }
            drop(store);
            flash.0.borrow_mut().fail = None;
            let mut reopened = VaultStore::new(flash, 31).unwrap();
            if let Ok(recovered) = run(reopened.recover()) {
                if ok {
                    assert_eq!(recovered, &after);
                } else {
                    assert!(recovered == &before || recovered == &after);
                }
            } else {
                assert!(!ok, "acknowledged revoke not recoverable at op {at}");
            }
        }
    }
    println!("verified {operations} storage operations x 3 fault modes");
}

#[test]
fn corrupted_committed_metadata_cannot_resurrect_revoked_peer() {
    let flash = Flash::from(bound_image());
    let mut store = VaultStore::new(flash.clone(), 13).unwrap();
    let revoke = run(store.recover()).unwrap().revoke([1; 16]).unwrap();
    run(store.commit(&revoke)).unwrap();
    for _ in 0..40 {
        let enrolled = store.current().unwrap().enroll(binding(1)).unwrap();
        run(store.commit(&enrolled)).unwrap();
        let revoked = store.current().unwrap().revoke([1; 16]).unwrap();
        run(store.commit(&revoked)).unwrap();
    }
    let image = flash.0.borrow().bytes.clone();
    for page in 0..image.len() / PAGE_BYTES {
        if image[page * PAGE_BYTES..(page + 1) * PAGE_BYTES]
            .iter()
            .all(|b| *b == 0xff)
        {
            continue;
        }
        for byte in 0..32 {
            let mut corrupt = image.clone();
            corrupt[page * PAGE_BYTES + byte] ^= 1;
            let mut store = VaultStore::new(Flash::from(corrupt), 0).unwrap();
            if let Ok(vault) = run(store.recover()) {
                assert!(
                    vault.find([1; 16]).is_none(),
                    "old binding resurrected: page {page}, byte {byte}"
                );
            }
        }
    }
}

#[test]
fn wrong_header_and_dirty_uninitialized_storage_never_formats() {
    for index in [0, 255, 256, PAGE_BYTES, PAGE_BYTES * 5] {
        let flash = Flash::new();
        flash.0.borrow_mut().bytes[index] = 0;
        let mut store = VaultStore::new(flash.clone(), 0).unwrap();
        assert!(run(store.recover()).is_err());
        assert!(matches!(
            run(store.initialize(local())),
            Err(Error::Io(IoError::Code(Code::NotBlank)))
        ));
        assert_eq!(flash.0.borrow().writes, 0);
    }
}
