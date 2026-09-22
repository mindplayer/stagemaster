use stagemaster_project::{Document, EditCommand};
use stagemaster_project_store::DiskFile;
use std::{fs, path::PathBuf};
fn directory() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
#[test]
fn save_reopen_changes_revision_and_retains_all_content() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    let mut file = DiskFile::select(&path).unwrap();
    let original = Document::new("演出").unwrap();
    let first = file.save(&original).unwrap().document;
    let (read, mut reopened) = DiskFile::open(&path).unwrap();
    assert_eq!(first, read);
    assert!(read.same_content(&original));
    let mut edited = read;
    edited
        .edit(EditCommand::SetInfo {
            name: "演出二".into(),
            description: "现场".into(),
        })
        .unwrap();
    let second = reopened.save(&edited).unwrap().document;
    assert_eq!(DiskFile::open(&path).unwrap().0, second);
    assert!(second.same_content(&edited));
    assert_ne!(first, second);
}
#[test]
fn external_change_and_deletion_are_never_silently_overwritten() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    let doc = Document::new("工程").unwrap();
    let mut file = DiskFile::select(&path).unwrap();
    file.save(&doc).unwrap();
    fs::write(&path, b"external content").unwrap();
    assert!(file.save(&doc).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"external content");
    fs::remove_file(&path).unwrap();
    assert!(file.save(&doc).is_err());
    assert!(!path.exists());
}
#[test]
fn stale_session_cannot_overwrite_a_new_revision() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    let doc = Document::new("工程").unwrap();
    DiskFile::select(&path).unwrap().save(&doc).unwrap();
    let (_, mut a) = DiskFile::open(&path).unwrap();
    let (_, mut b) = DiskFile::open(&path).unwrap();
    a.save(&doc).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(b.save(&doc).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}
#[test]
fn failed_precommit_preserves_original_and_can_retry() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    let doc = Document::new("工程").unwrap();
    let mut file = DiskFile::select(&path).unwrap();
    file.save(&doc).unwrap();
    let bytes = fs::read(&path).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.path().join("工程.json.stagemaster-lock"))
        .unwrap();
    lock.lock().unwrap();
    assert!(file.save(&doc).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(lock);
    assert!(file.save(&doc).is_ok());
}
#[test]
fn new_destination_conflict_preserves_existing_file() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    let mut file = DiskFile::select(&path).unwrap();
    fs::write(&path, b"appeared").unwrap();
    assert!(file.save(&Document::new("工程").unwrap()).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"appeared");
}
#[cfg(unix)]
#[test]
fn symbolic_links_and_directories_are_rejected() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    fs::write(&path, Document::new("工程").unwrap().encode().unwrap()).unwrap();
    let link = dir.path().join("link.json");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(DiskFile::open(&link).is_err());
    assert!(DiskFile::select(dir.path()).is_err());
}
