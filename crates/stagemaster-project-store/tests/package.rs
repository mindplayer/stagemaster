use stagemaster_project::{Document, PackageSelection};
use stagemaster_project_store::PackageFile;
use std::{fs, path::PathBuf};
fn dir() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
fn package() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    value["entryPoints"] = serde_json::json!([]);
    let d = Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    d.build_package(&[PackageSelection::Scene {
        id: d.view().scenes[0].id.clone(),
    }])
    .unwrap()
    .bytes
}
#[test]
fn exports_are_atomic_and_do_not_touch_source_or_accept_other_files() {
    let dir = dir();
    let source = dir.path().join("原工程.smpkg");
    let original = Document::new("原工程").unwrap().encode().unwrap();
    fs::write(&source, &original).unwrap();
    assert!(PackageFile::select(&source, Some(&source)).is_err());
    assert!(PackageFile::select(&source, None).is_err());
    assert!(PackageFile::select(&dir.path().join("不能.json"), None).is_err());
    let path = dir.path().join("节目.smpkg");
    let bytes = package();
    let mut selected = PackageFile::select(&path, Some(&source)).unwrap();
    assert!(!path.exists());
    assert!(selected.save(&bytes[..80]).is_err());
    assert!(!path.exists());
    assert_eq!(selected.save(&bytes).unwrap(), None);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::read(&source).unwrap(), original);
    PackageFile::select(&path, Some(&source))
        .unwrap()
        .save(&bytes)
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), bytes);
}
#[test]
fn changed_or_new_destinations_cannot_be_clobbered_and_failures_can_retry() {
    let dir = dir();
    let path = dir.path().join("节目.smpkg");
    let bytes = package();
    let mut selected = PackageFile::select(&path, None).unwrap();
    fs::write(&path, b"other").unwrap();
    assert!(selected.save(&bytes).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"other");
    fs::remove_file(&path).unwrap();
    selected.save(&bytes).unwrap();
    fs::write(&path, b"changed").unwrap();
    assert!(selected.save(&bytes).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"changed");
}
#[cfg(unix)]
#[test]
fn package_export_refuses_links_even_when_the_target_is_valid() {
    let dir = dir();
    let target = dir.path().join("目标.smpkg");
    let link = dir.path().join("链接.smpkg");
    fs::write(&target, package()).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(PackageFile::select(&link, None).is_err());
}
