use stagemaster_project::Document;
use stagemaster_project_store::SequenceReportFile;
use std::{fs, path::PathBuf};
fn report() -> stagemaster_project::SequenceReport {
    let mut root: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = serde_json::json!([]);
    let doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    doc.sequence_report(&doc.view().sequences[0].id).unwrap()
}
fn dir() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
#[test]
fn exports_without_project_revision_and_refuses_unrelated_files_and_source() {
    let dir = dir();
    let doc = Document::new("交接工程").unwrap();
    let report = report();
    let source = dir.path().join("工程.csv");
    let before = doc.encode().unwrap();
    fs::write(&source, &before).unwrap();
    assert!(SequenceReportFile::select(&source, Some(&source)).is_err());
    assert!(SequenceReportFile::select(&source, None).is_err());
    assert!(SequenceReportFile::select(&dir.path().join("错误.json"), None).is_err());
    let other = dir.path().join("其他.csv");
    fs::write(&other, b"name,address\r\nlight,1\r\n").unwrap();
    assert!(SequenceReportFile::select(&other, None).is_err());
    let target = dir.path().join("配灯表.csv");
    let mut file = SequenceReportFile::select(&target, Some(&source)).unwrap();
    assert!(!target.exists());
    assert_eq!(file.save(&report).unwrap(), None);
    assert_eq!(fs::read(&target).unwrap(), report.bytes());
    assert_eq!(doc.encode().unwrap(), before);
    assert_eq!(fs::read(&source).unwrap(), before);
    SequenceReportFile::select(&target, None)
        .unwrap()
        .save(&report)
        .unwrap();
}
#[test]
fn competing_changes_and_writers_do_not_replace_existing_data() {
    let dir = dir();
    let path = dir.path().join("资料.csv");
    let report = report();
    let mut file = SequenceReportFile::select(&path, None).unwrap();
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
    let report = report();
    fs::write(&target, report.bytes()).unwrap();
    let mut before = SequenceReportFile::select(&link, None).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(before.save(&report).is_err());
    assert!(SequenceReportFile::select(&link, None).is_err());
    assert_eq!(fs::read(target).unwrap(), report.bytes());
}

#[test]
fn report_types_refuse_to_replace_each_other() {
    let dir = dir();
    let path = dir.path().join("资料.csv");
    let fixture_report = Document::new("资料").unwrap().patch_report().unwrap();
    fs::write(&path, fixture_report.bytes()).unwrap();
    assert!(SequenceReportFile::select(&path, None).is_err());
    fs::write(&path, report().bytes()).unwrap();
    assert!(stagemaster_project_store::PatchReportFile::select(&path, None).is_err());
}
