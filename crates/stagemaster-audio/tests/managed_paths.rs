#![cfg(unix)]
use stagemaster_audio::{ResourceFileHealth, ResourceSource, Resources, verify};
use std::{fs, os::unix::fs::symlink, path::PathBuf, sync::atomic::AtomicBool};

fn directory() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
fn import_fixture(root: &std::path::Path) -> (Resources, String, PathBuf) {
    let source = root.join("original.wav");
    fs::write(
        &source,
        b"managed resource identity fixture, not decoded audio",
    )
    .unwrap();
    let resources = Resources::new(root.join("cache"));
    let (digest, cached) = resources
        .import(&source, "wav", None, &AtomicBool::new(false))
        .unwrap();
    (resources, digest, cached)
}
fn invalid(value: &ResourceFileHealth) {
    assert!(
        matches!(value, ResourceFileHealth::Invalid { .. }),
        "unsafe managed path was reported as {value:?}"
    );
}
#[test]
fn linked_companion_directory_is_not_portable_and_is_not_written() {
    let temp = directory();
    let (resources, digest, _) = import_fixture(temp.path());
    let project = temp.path().join("show.json");
    fs::write(&project, b"original project bytes").unwrap();
    let outside = temp.path().join("not-owned-companion");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, temp.path().join("show.json.assets")).unwrap();
    let result = resources.archive(&digest, "wav", None, &project);
    let unexpected_entries = fs::read_dir(&outside).unwrap().count();
    assert!(
        result.is_err(),
        "archive result={result:?}, entries written through link={unexpected_entries}"
    );
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    assert_eq!(fs::read(&project).unwrap(), b"original project bytes");
}
#[test]
fn content_correct_linked_file_is_neither_reused_nor_reported_valid() {
    let temp = directory();
    let (resources, digest, cached) = import_fixture(temp.path());
    let project = temp.path().join("show.json");
    let companion = temp.path().join("show.json.assets");
    fs::create_dir(&companion).unwrap();
    let path = companion.join(format!("{digest}.wav"));
    let original = fs::read(&cached).unwrap();
    symlink(&cached, &path).unwrap();
    let health = resources
        .inspect(&digest, "wav", Some(&project), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(health.local, ResourceFileHealth::Valid);
    invalid(&health.companion);
    assert!(resources.archive(&digest, "wav", None, &project).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&cached).unwrap(), original);
}
#[test]
fn linked_cache_root_is_rejected_before_creation() {
    linked_cache_directory(false);
}
#[test]
fn linked_cache_ancestor_is_rejected_before_creation() {
    linked_cache_directory(true);
}
fn linked_cache_directory(ancestor: bool) {
    let temp = directory();
    let source = temp.path().join("source.wav");
    fs::write(&source, b"external source bytes").unwrap();
    let outside = temp.path().join("other-directory");
    fs::create_dir(&outside).unwrap();
    let link = temp.path().join("linked-root");
    symlink(&outside, &link).unwrap();
    let root = if ancestor {
        link.join("new-cache")
    } else {
        link
    };
    let result = Resources::new(root).import(&source, "wav", None, &AtomicBool::new(false));
    let unexpected_entries = fs::read_dir(&outside).unwrap().count();
    assert!(
        result.is_err(),
        "accepted linked directory, ancestor={ancestor}, entries written through link={unexpected_entries}"
    );
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
}
#[test]
fn linked_cache_file_is_rejected_even_when_companion_is_healthy() {
    let temp = directory();
    let (resources, digest, cached) = import_fixture(temp.path());
    let project = temp.path().join("show.json");
    resources.archive(&digest, "wav", None, &project).unwrap();
    let companion = temp
        .path()
        .join("show.json.assets")
        .join(format!("{digest}.wav"));
    fs::remove_file(&cached).unwrap();
    symlink(&companion, &cached).unwrap();
    assert!(resources.resolve(&digest, "wav", Some(&project)).is_err());
    let health = resources
        .inspect(&digest, "wav", Some(&project), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(health.local_source, Some(ResourceSource::Cache));
    invalid(&health.local);
    assert_eq!(health.companion, ResourceFileHealth::Valid);
    assert!(
        resources
            .import(
                &temp.path().join("original.wav"),
                "wav",
                Some(&digest),
                &AtomicBool::new(false)
            )
            .is_err()
    );
}
#[test]
fn dangling_managed_link_is_invalid_and_is_not_silently_repaired() {
    let temp = directory();
    let (resources, digest, _) = import_fixture(temp.path());
    let project = temp.path().join("show.json");
    let companion = temp.path().join("show.json.assets");
    fs::create_dir(&companion).unwrap();
    let path = companion.join(format!("{digest}.wav"));
    let absent = temp.path().join("absent.wav");
    symlink(&absent, &path).unwrap();
    let health = resources
        .inspect(&digest, "wav", Some(&project), &AtomicBool::new(false))
        .unwrap();
    invalid(&health.companion);
    assert!(resources.archive(&digest, "wav", None, &project).is_err());
    assert!(
        fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!absent.exists());
}
#[test]
fn explicit_source_link_is_copied_into_a_plain_managed_file() {
    let temp = directory();
    let (_, digest, original) = import_fixture(temp.path());
    let selected = temp.path().join("selected.wav");
    symlink(&original, &selected).unwrap();
    let (actual, managed) = Resources::new(temp.path().join("second-cache"))
        .import(&selected, "wav", Some(&digest), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(actual, digest);
    assert!(fs::symlink_metadata(&managed).unwrap().is_file());
    verify(&managed, &digest, &AtomicBool::new(false)).unwrap();
}
#[test]
fn ordinary_corruption_can_still_be_repaired_without_changing_the_source() {
    let temp = directory();
    let (resources, digest, cached) = import_fixture(temp.path());
    let original = fs::read(temp.path().join("original.wav")).unwrap();
    let project = temp.path().join("show.json");
    resources.archive(&digest, "wav", None, &project).unwrap();
    let companion = temp
        .path()
        .join("show.json.assets")
        .join(format!("{digest}.wav"));
    fs::write(&companion, b"bad ordinary copy").unwrap();
    resources.archive(&digest, "wav", None, &project).unwrap();
    verify(&companion, &digest, &AtomicBool::new(false)).unwrap();
    fs::write(&cached, b"bad ordinary cache").unwrap();
    resources
        .import(
            &temp.path().join("original.wav"),
            "wav",
            Some(&digest),
            &AtomicBool::new(false),
        )
        .unwrap();
    verify(&cached, &digest, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        fs::read(temp.path().join("original.wav")).unwrap(),
        original
    );
}
#[test]
fn repairing_an_ordinary_hardlinked_cache_replaces_only_the_managed_name() {
    let temp = directory();
    let (resources, digest, cached) = import_fixture(temp.path());
    let other = temp.path().join("other-name.wav");
    fs::write(&other, b"other existing bytes").unwrap();
    fs::remove_file(&cached).unwrap();
    fs::hard_link(&other, &cached).unwrap();
    resources
        .import(
            &temp.path().join("original.wav"),
            "wav",
            Some(&digest),
            &AtomicBool::new(false),
        )
        .unwrap();
    verify(&cached, &digest, &AtomicBool::new(false)).unwrap();
    assert_eq!(fs::read(&other).unwrap(), b"other existing bytes");
}
#[test]
fn a_shared_cache_and_companion_path_keeps_both_health_views_consistent() {
    let temp = directory();
    let (_, digest, original) = import_fixture(temp.path());
    let project = temp.path().join("shared.json");
    let resources = Resources::new(temp.path().join("shared.json.assets"));
    resources
        .import(&original, "wav", Some(&digest), &AtomicBool::new(false))
        .unwrap();
    let health = resources
        .inspect(&digest, "wav", Some(&project), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(health.local_source, Some(ResourceSource::Cache));
    assert_eq!(health.local, ResourceFileHealth::Valid);
    assert_eq!(health.companion, ResourceFileHealth::Valid);
}
