#[path = "support/fixture.rs"]
mod fixture;
use stagemaster_delivery::{Error, Incoming, Package, from_file};
use stagemaster_package::{MAX_PACKAGE_BYTES, ReadAt};
use std::{fs, sync::atomic::AtomicBool};

#[test]
fn local_content_is_independent_of_path_and_removable_source() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("first.smpkg");
    let b = dir.path().join("renamed.smpkg");
    let bytes = fixture::bytes(12000);
    fs::write(&a, &bytes).unwrap();
    fs::write(&b, &bytes).unwrap();
    let cancel = AtomicBool::new(false);
    let first = from_file(&a, None, &cancel).unwrap();
    let second = from_file(&b, Some(first.identity()), &cancel).unwrap();
    fs::remove_file(a).unwrap();
    fs::remove_file(b).unwrap();
    assert_eq!(first.identity(), second.identity());
    let mut read = vec![0; bytes.len()];
    second.read_exact(0, &mut read).unwrap();
    assert_eq!(read, bytes);
    assert_eq!(first.archive().source(), second.archive().source());
}
#[test]
fn incomplete_extra_wrong_identity_and_corrupt_content_never_finish() {
    let bytes = fixture::bytes(12000);
    let expected = Package::from_bytes(bytes.clone().into(), None)
        .unwrap()
        .identity();
    let mut part = Incoming::new(Some(expected)).unwrap();
    part.push(&bytes[..100]).unwrap();
    assert_eq!(part.received(), 100);
    assert!(matches!(part.finish(), Err(Error::Incomplete)));
    let mut extra = Incoming::new(Some(expected)).unwrap();
    extra.push(&bytes).unwrap();
    assert!(matches!(extra.push(&[0]), Err(Error::Bounds)));
    assert!(matches!(extra.finish(), Err(Error::Bounds)));
    let mut wrong = expected;
    wrong.digest[0] ^= 1;
    let mut content = Incoming::new(Some(wrong)).unwrap();
    content.push(&bytes).unwrap();
    assert!(matches!(content.finish(), Err(Error::Identity)));
    let mut corrupt = bytes;
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(Package::from_bytes(corrupt.into(), Some(expected)).is_err());
    let mut huge = Incoming::new(None).unwrap();
    assert!(matches!(
        huge.push(&vec![0; MAX_PACKAGE_BYTES + 1]),
        Err(Error::Bounds)
    ));
    assert!(huge.finish().is_err());
    wrong.bytes = usize::MAX;
    assert!(matches!(Incoming::new(Some(wrong)), Err(Error::Bounds)));
}
#[test]
fn cancelled_and_non_regular_local_sources_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("source.smpkg");
    fs::write(&file, fixture::bytes(12000)).unwrap();
    assert!(matches!(
        from_file(&file, None, &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        from_file(dir.path(), None, &AtomicBool::new(false)),
        Err(Error::Source)
    ));
    #[cfg(unix)]
    {
        let alias = dir.path().join("alias.smpkg");
        std::os::unix::fs::symlink(&file, &alias).unwrap();
        assert!(matches!(
            from_file(&alias, None, &AtomicBool::new(false)),
            Err(Error::Source)
        ));
    }
    let sparse = fs::File::create(file.clone()).unwrap();
    sparse.set_len((MAX_PACKAGE_BYTES + 1) as u64).unwrap();
    assert!(matches!(
        from_file(&file, None, &AtomicBool::new(false)),
        Err(Error::Bounds)
    ));
}
