use super::*;
fn temporary() -> tempfile::TempDir {
    tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
fn owner(root: &Path) -> crate::storage_paths::Directories {
    crate::storage_paths::Directories {
        data: root.join("data"),
        temporary: root.join("tmp"),
        logs: root.join("logs"),
    }
}

#[test]
fn installed_runtime_has_its_own_user_cache_logs_and_temporary_namespace() {
    let temp = temporary();
    let paths = RuntimePaths::component(&owner(temp.path()));
    paths.prepare().unwrap();
    assert_eq!(paths.root, temp.path().join("data/previs"));
    assert_eq!(paths.user, paths.root.join("user"));
    assert_eq!(paths.cache, paths.root.join("cache"));
    assert_eq!(paths.temporary, temp.path().join("tmp/previs"));
    assert_eq!(paths.logs, temp.path().join("logs/previs"));
    assert!(!temp.path().join("data/navigation").exists());
    assert!(!temp.path().join("data/recovery").exists());
}

#[test]
fn editor_paths_preserve_the_existing_project_cache_graph() {
    let temp = temporary();
    let paths = RuntimePaths::editor(temp.path().to_owned());
    assert_eq!(paths.logs, temp.path().join("logs"));
    assert_eq!(paths.temporary, temp.path().join("tmp"));
    assert_eq!(paths.user, temp.path().join("data/previs-user"));
    assert_eq!(paths.cache, temp.path().join("data/previs-derived-cache"));
}

#[test]
fn separate_acceptance_owners_never_share_renderer_or_signal_logs() {
    let temp = temporary();
    let a = RuntimePaths::component(&owner(&temp.path().join("a")));
    let b = RuntimePaths::component(&owner(&temp.path().join("b")));
    a.prepare().unwrap();
    b.prepare().unwrap();
    fs::write(a.logs.join("previs-signalling.log"), b"a").unwrap();
    fs::write(b.logs.join("previs-signalling.log"), b"b").unwrap();
    for (a, b) in [
        (&a.root, &b.root),
        (&a.logs, &b.logs),
        (&a.temporary, &b.temporary),
        (&a.user, &b.user),
        (&a.cache, &b.cache),
    ] {
        assert_ne!(a, b);
    }
    assert_eq!(
        fs::read(a.logs.join("previs-signalling.log")).unwrap(),
        b"a"
    );
    assert_eq!(
        fs::read(b.logs.join("previs-signalling.log")).unwrap(),
        b"b"
    );
}

#[test]
fn a_conflicting_directory_refuses_before_creating_other_folders() {
    let temp = temporary();
    let paths = RuntimePaths::component(&owner(temp.path()));
    fs::create_dir_all(paths.cache.parent().unwrap()).unwrap();
    fs::write(&paths.cache, b"keep").unwrap();
    assert!(paths.prepare().is_err());
    assert_eq!(fs::read(paths.cache).unwrap(), b"keep");
    for path in [&paths.logs, &paths.temporary, &paths.user] {
        assert!(!path.exists());
    }
}

#[cfg(unix)]
#[test]
fn linked_parent_or_final_directory_is_refused_before_any_new_writes() {
    use std::os::unix::fs::symlink;
    let temp = temporary();
    let outside = temp.path().join("other-owner");
    fs::create_dir(&outside).unwrap();
    for final_directory in [false, true] {
        let root = temp
            .path()
            .join(if final_directory { "final" } else { "parent" });
        fs::create_dir(&root).unwrap();
        let paths = RuntimePaths::component(&owner(&root));
        if final_directory {
            fs::create_dir_all(paths.cache.parent().unwrap()).unwrap();
            symlink(&outside, &paths.cache).unwrap();
        } else {
            symlink(&outside, root.join("data")).unwrap();
        }
        assert!(paths.prepare().is_err());
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
        assert!(!paths.logs.exists() && !paths.temporary.exists() && !paths.user.exists());
    }
}

#[cfg(unix)]
#[test]
fn a_linked_foundation_user_refuses_before_other_instance_directories_are_created() {
    let temp = temporary();
    let paths = RuntimePaths::component(&owner(temp.path()));
    fs::create_dir_all(&paths.user).unwrap();
    let outside = temp.path().join("other-owner");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, paths.user.join("platform-user")).unwrap();
    assert!(paths.prepare().is_err());
    assert!(!paths.logs.exists() && !paths.temporary.exists() && !paths.cache.exists());
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}
