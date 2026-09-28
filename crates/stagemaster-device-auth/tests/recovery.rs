#![cfg(feature = "persistence")]
mod support;
use stagemaster_device_auth::persistence::VaultStore;
use support::{Failure, Flash, binding, local, run};

#[test]
fn physical_read_failure_cannot_be_silently_skipped_during_authority_recovery() {
    let flash = Flash::new();
    let mut store = VaultStore::new(flash.clone(), 5).unwrap();
    run(store.initialize(local())).unwrap();
    for _ in 0..40 {
        let add = store.current().unwrap().enroll(binding(1)).unwrap();
        run(store.commit(&add)).unwrap();
        let revoke = store.current().unwrap().revoke([1; 16]).unwrap();
        run(store.commit(&revoke)).unwrap();
    }
    let image = flash.0.borrow().bytes.clone();
    drop(store);
    let mut store = VaultStore::new(flash.clone(), 11).unwrap();
    flash.0.borrow_mut().ops = 0;
    run(store.recover()).unwrap();
    let reads = flash.0.borrow().ops;
    for at in 1..=reads {
        let flash = Flash::from(image.clone());
        flash.0.borrow_mut().fail = Some((at, Failure::Before));
        let mut store = VaultStore::new(flash.clone(), 11).unwrap();
        assert!(run(store.recover()).is_err(), "read {at} was skipped");
        assert!(store.current().is_none());
        flash.0.borrow_mut().fail = None;
        assert!(run(store.recover()).unwrap().find([1; 16]).is_none());
    }
    println!("verified {reads} recovery read failures");
}

#[test]
fn interrupted_initialization_never_formats_existing_bytes_or_returns_partial_identity() {
    let flash = Flash::new();
    let mut store = VaultStore::new(flash.clone(), 5).unwrap();
    let initial = run(store.initialize(local())).unwrap().clone();
    let operations = flash.0.borrow().ops;
    for at in 1..=operations {
        for failure in [Failure::Before, Failure::Partial, Failure::After] {
            let flash = Flash::new();
            flash.0.borrow_mut().fail = Some((at, failure));
            let mut store = VaultStore::new(flash.clone(), 5).unwrap();
            let initialized = run(store.initialize(local())).is_ok();
            if !initialized {
                assert!(store.current().is_none());
            }
            flash.0.borrow_mut().fail = None;
            let image = flash.0.borrow().bytes.clone();
            drop(store);
            let mut store = VaultStore::new(flash.clone(), 2).unwrap();
            if let Ok(record) = run(store.recover()) {
                assert_eq!(record, &initial);
            } else {
                assert!(!initialized);
            }
            if image.iter().any(|b| *b != 0xff) {
                let writes = flash.0.borrow().writes;
                assert!(run(store.initialize(local())).is_err());
                assert_eq!(flash.0.borrow().writes, writes);
            }
        }
    }
    println!("verified {operations} initialization operations x 3 fault modes");
}
