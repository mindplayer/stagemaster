use stagemaster_project::Document;
use stagemaster_project_store::PatchReportFile;
use std::{fs, path::PathBuf};
fn dir() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
#[test]
fn exports_without_project_revision_and_refuses_unrelated_files_and_source() {
    let dir = dir();
    let doc = Document::new("交接工程").unwrap();
    let report = doc.patch_report().unwrap();
    let source = dir.path().join("工程.csv");
    let before = doc.encode().unwrap();
    fs::write(&source, &before).unwrap();
    assert!(PatchReportFile::select(&source, Some(&source)).is_err());
    assert!(PatchReportFile::select(&source, None).is_err());
    assert!(PatchReportFile::select(&dir.path().join("错误.json"), None).is_err());
    let other = dir.path().join("其他.csv");
    fs::write(&other, b"name,address\r\nlight,1\r\n").unwrap();
    assert!(PatchReportFile::select(&other, None).is_err());
    let target = dir.path().join("配灯表.csv");
    let mut file = PatchReportFile::select(&target, Some(&source)).unwrap();
    assert!(!target.exists());
    assert_eq!(file.save(&report).unwrap(), None);
    assert_eq!(fs::read(&target).unwrap(), report.bytes());
    assert_eq!(doc.encode().unwrap(), before);
    assert_eq!(fs::read(&source).unwrap(), before);
    PatchReportFile::select(&target, None)
        .unwrap()
        .save(&report)
        .unwrap();
}
#[test]
fn competing_changes_and_writers_do_not_replace_existing_data() {
    let dir = dir();
    let path = dir.path().join("资料.csv");
    let report = Document::new("资料").unwrap().patch_report().unwrap();
    let mut file = PatchReportFile::select(&path, None).unwrap();
    fs::write(&path, b"concurrent writer").unwrap();
    assert!(file.save(&report).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"concurrent writer");
    fs::remove_file(&path).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.path().join("资料.csv.stagemaster-lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(file.save(&report).is_err());
    assert!(!path.exists());
    fs::File::unlock(&lock).unwrap();
    file.save(&report).unwrap();
    assert_eq!(fs::read(&path).unwrap(), report.bytes());
}
#[cfg(unix)]
#[test]
fn symbolic_links_are_refused_before_and_after_selection() {
    let dir = dir();
    let target = dir.path().join("目标.csv");
    let link = dir.path().join("链接.csv");
    let report = Document::new("资料").unwrap().patch_report().unwrap();
    fs::write(&target, report.bytes()).unwrap();
    let mut before = PatchReportFile::select(&link, None).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(before.save(&report).is_err());
    assert!(PatchReportFile::select(&link, None).is_err());
    assert_eq!(fs::read(target).unwrap(), report.bytes());
}
