use crate::OutputScope;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn ownership_is_scoped_and_uses_a_persistent_private_file() {
    let dir = tempfile::tempdir().unwrap();
    let scope = OutputScope::new(dir.path().join("a")).unwrap();
    let other = OutputScope::new(dir.path().join("b")).unwrap();
    let lease = scope.reserve().unwrap();
    let alias = OutputScope::new(dir.path().join("a").join(".")).unwrap();
    assert!(scope.reserve().is_err());
    assert!(alias.reserve().is_err());
    assert!(other.reserve().is_ok());
    let path = dir.path().join("a/audio-output.lock");
    fs::write(&path, b"existing marker").unwrap();
    drop(lease);
    let _new = scope.reserve().unwrap();
    assert_eq!(fs::read(path).unwrap(), b"existing marker");
}

#[test]
fn relative_public_and_symlinked_coordination_paths_are_rejected() {
    assert!(OutputScope::new("relative".into()).is_err());
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("route");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    let scope = OutputScope::new(root.clone()).unwrap();
    assert!(scope.reserve().is_err());
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let alias = dir.path().join("alias");
    symlink(&root, &alias).unwrap();
    assert!(OutputScope::new(alias).unwrap().reserve().is_err());
    let target = dir.path().join("target");
    fs::write(&target, b"unchanged").unwrap();
    let lock = root.join("audio-output.lock");
    symlink(&target, &lock).unwrap();
    assert!(scope.reserve().is_err());
    assert_eq!(fs::read(target).unwrap(), b"unchanged");
    fs::remove_file(&lock).unwrap();
    fs::write(&lock, b"marker").unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(scope.reserve().is_err());
}
