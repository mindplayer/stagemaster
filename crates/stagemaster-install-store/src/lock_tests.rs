// Tests deliberately duplicate the private RAII handles to reproduce incidental inheritance.
#![allow(clippy::used_underscore_binding)]
use super::*;
#[cfg(unix)]
#[test]
fn writer_lifetime_ends_before_an_unowned_duplicate_descriptor_closes() {
    let dir = super::tests::dir();
    let store = FileStore::open(dir.path()).unwrap();
    let inherited = store._writer.duplicate().unwrap();
    assert!(FileStore::open(dir.path()).is_err());
    drop(store);
    let reopened = FileStore::open(dir.path())
        .expect("store drop must end its writer lease even while a duplicated descriptor is alive");
    assert!(FileStore::open(dir.path()).is_err());
    drop(inherited);
    assert!(FileStore::open(dir.path()).is_err());
    drop(reopened);
    assert!(FileStore::open(dir.path()).is_ok());
}
#[cfg(unix)]
#[test]
fn slot_leases_keep_independent_readers_and_release_only_the_departing_owner() {
    let dir = super::tests::dir();
    let mut store = FileStore::open(dir.path()).unwrap();
    fs::write(store.payload(Slot::A), b"payload").unwrap();
    let first = store.snapshot(Slot::A).unwrap();
    let second = store.snapshot(Slot::A).unwrap();
    let incidental = first._lease.duplicate().unwrap();
    assert!(store.prepare(Slot::A, 3).is_err());
    drop(first);
    assert!(store.prepare(Slot::A, 3).is_err());
    drop(second);
    store.prepare(Slot::A, 3).unwrap();
    let staged = store.stage.as_ref().unwrap()._lease.duplicate().unwrap();
    assert!(store.snapshot(Slot::A).is_err());
    store.release();
    assert!(store.snapshot(Slot::A).is_ok());
    drop((incidental, staged));
}
