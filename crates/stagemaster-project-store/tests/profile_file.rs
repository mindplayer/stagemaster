use stagemaster_project::{Document, MAX_PROFILE_FILE_BYTES};
use stagemaster_project_store::ProfileFileStore;
use std::{fs, path::PathBuf};
fn dir() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
#[test]
fn save_read_replace_and_conflict_preserve_both_project_and_mode_identity() {
    let dir = dir();
    let d = Document::new("灯库").unwrap();
    let profile = d.profile_file(&d.view().profiles[1].id).unwrap();
    let before = d.encode().unwrap();
    let path = dir.path().join("灯.smfixture.json");
    let mut file = ProfileFileStore::select(&path).unwrap();
    file.save(&profile).unwrap();
    assert_eq!(
        ProfileFileStore::read(&path).unwrap().encode().unwrap(),
        profile.encode().unwrap()
    );
    file.save(&profile).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.path().join("灯.smfixture.json.stagemaster-lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(file.save(&profile).is_err());
    fs::File::unlock(&lock).unwrap();
    fs::write(&path, b"concurrent writer").unwrap();
    assert!(file.save(&profile).is_err());
    assert_eq!(fs::read(path).unwrap(), b"concurrent writer");
    assert_eq!(d.encode().unwrap(), before);
}
#[test]
fn unrelated_project_and_oversized_input_are_never_treated_as_modes() {
    let dir = dir();
    let path = dir.path().join("项目.smfixture.json");
    let before = Document::new("不能覆盖").unwrap().encode().unwrap();
    fs::write(&path, &before).unwrap();
    assert!(ProfileFileStore::select(&path).is_err());
    assert!(ProfileFileStore::read(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(ProfileFileStore::select(&dir.path().join("模式.json")).is_err());
    let large = dir.path().join("超限.smfixture.json");
    fs::File::create(&large)
        .unwrap()
        .set_len((MAX_PROFILE_FILE_BYTES + 1) as u64)
        .unwrap();
    assert!(
        ProfileFileStore::read(&large)
            .err()
            .unwrap()
            .contains("512 KiB")
    );
    assert!(ProfileFileStore::read(dir.path()).is_err());
}
#[cfg(unix)]
#[test]
fn linked_destinations_and_sources_are_refused() {
    let dir = dir();
    let path = dir.path().join("原始.smfixture.json");
    let link = dir.path().join("链接.smfixture.json");
    let d = Document::new("链接").unwrap();
    let profile = d.profile_file(&d.view().profiles[0].id).unwrap();
    fs::write(&path, profile.encode().unwrap()).unwrap();
    let mut captured = ProfileFileStore::select(&link).unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(ProfileFileStore::read(&link).is_err());
    assert!(ProfileFileStore::select(&link).is_err());
    assert!(captured.save(&profile).is_err());
    assert_eq!(fs::read(path).unwrap(), profile.encode().unwrap());
}
