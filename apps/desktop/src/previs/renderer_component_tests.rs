use super::*;
use std::fs;

const CONTEXT: &[&str] = &[
    "package.json",
    "package-lock.json",
    "node_modules/@epicgames-ps/lib-pixelstreamingcommon-ue5.8/package.json",
    "node_modules/@epicgames-ps/lib-pixelstreamingcommon-ue5.8/dist/cjs/pixelstreamingcommon.js",
    "node_modules/@epicgames-ps/lib-pixelstreamingsignalling-ue5.8/package.json",
    "node_modules/@epicgames-ps/lib-pixelstreamingsignalling-ue5.8/dist/cjs/pixelstreamingsignalling.js",
];

fn temporary() -> tempfile::TempDir {
    tempfile::tempdir_in(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}

fn file(path: &Path, content: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn complete(base: &Path) {
    file(&executable(base), b"binary fixture, not a runnable Game");
    file(
        &base.join(if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "node"
        }),
        b"node fixture",
    );
    file(&base.join("signalling.mjs"), b"export {};");
    for entry in CONTEXT {
        file(&base.join(entry), b"{}");
    }
    if cfg!(target_os = "macos") {
        file(
            &base.join("StageMasterPreview.app/Contents/Info.plist"),
            b"fixture",
        );
        file(&base.join("StageMasterPreview.app/Contents/UE/StageMasterPreview/Content/Paks/StageMasterPreview-Mac.pak"), b"fixture");
    }
}

#[test]
fn only_an_absent_component_allows_the_development_editor_fallback() {
    let temp = temporary();
    let base = temp.path().join("previs");
    assert!(installed(&base).unwrap().is_none());
    fs::create_dir(&base).unwrap();
    assert!(
        installed(&base).is_err(),
        "partial install cannot masquerade as absent"
    );
}

#[test]
fn every_required_context_file_is_checked_before_starting_a_child() {
    let temp = temporary();
    let base = temp.path().join("previs");
    complete(&base);
    assert!(installed(&base).unwrap().is_some());
    for entry in CONTEXT {
        let path = base.join(entry);
        fs::rename(&path, path.with_extension("removed")).unwrap();
        assert!(installed(&base).is_err(), "missing {entry} was accepted");
        fs::rename(path.with_extension("removed"), path).unwrap();
    }
    assert!(installed(&base).unwrap().is_some());
}

#[test]
fn invalid_manifest_is_not_a_complete_component() {
    let temp = temporary();
    let base = temp.path().join("previs");
    complete(&base);
    for content in [
        b"not json".as_slice(),
        b"[]",
        b"null",
        &vec![b' '; 1_048_577],
    ] {
        file(&base.join("package.json"), content);
        assert!(installed(&base).is_err());
    }
    file(&base.join("package.json"), b"{}");
    assert!(installed(&base).unwrap().is_some());
}

#[test]
fn missing_entry_and_mac_resources_are_explicit_failures_not_an_absent_component() {
    let temp = temporary();
    let base = temp.path().join("previs");
    complete(&base);
    let mut paths = vec![
        executable(&base),
        base.join("signalling.mjs"),
        base.join(if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "node"
        }),
    ];
    if cfg!(target_os = "macos") {
        paths.push(base.join("StageMasterPreview.app/Contents/Info.plist"));
        paths.push(base.join("StageMasterPreview.app/Contents/UE/StageMasterPreview/Content/Paks/StageMasterPreview-Mac.pak"));
    }
    for path in paths {
        let original = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(installed(&base).is_err());
        file(&path, b"");
        assert!(installed(&base).is_err());
        file(&path, &original);
    }
    assert!(installed(&base).unwrap().is_some());
}

#[cfg(unix)]
#[test]
fn outside_component_links_and_non_executable_entries_are_rejected() {
    use std::os::unix::{fs::PermissionsExt, fs::symlink};
    let temp = temporary();
    let base = temp.path().join("previs");
    complete(&base);
    let program = executable(&base);
    fs::set_permissions(&program, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(installed(&base).is_err());
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    let script = base.join("signalling.mjs");
    fs::rename(&script, temp.path().join("outside.mjs")).unwrap();
    symlink(temp.path().join("outside.mjs"), script).unwrap();
    assert!(installed(&base).is_err());
    let linked = temp.path().join("linked-component");
    symlink(&base, &linked).unwrap();
    assert!(installed(&linked).is_err());
}
