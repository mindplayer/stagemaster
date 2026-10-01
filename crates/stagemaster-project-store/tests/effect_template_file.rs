use serde_json::json;
use stagemaster_project::{Document, EffectTemplateFile, MAX_EFFECT_TEMPLATE_BYTES};
use stagemaster_project_store::EffectTemplateFileStore;
use std::{fs, path::PathBuf};
fn file() -> EffectTemplateFile {
    EffectTemplateFile::create(
        serde_json::from_value(json!({
            "name": "模板",
            "recipe": {
                "kind": "intensity-wave", "waveform": "smooth",
                "low": 0, "high": 65535, "dutyPercent": 50
            },
            "timing": {
                "periodMs": 2000, "phaseDegrees": 0,
                "spreadDegrees": 0, "reverseOrder": false
            }
        }))
        .unwrap(),
    )
    .unwrap()
}
fn dir() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
#[test]
fn local_template_roundtrip_and_conflicting_writer_preserve_identity_and_bytes() {
    let dir = dir();
    let template = file();
    let path = dir.path().join("模板.smeffect.json");
    let mut target = EffectTemplateFileStore::select(&path).unwrap();
    target.save(&template).unwrap();
    assert_eq!(
        EffectTemplateFileStore::read(&path)
            .unwrap()
            .encode()
            .unwrap(),
        template.encode().unwrap()
    );
    target.save(&template).unwrap();
    fs::write(&path, b"changed").unwrap();
    assert!(target.save(&template).is_err());
    assert_eq!(fs::read(path).unwrap(), b"changed");
}
#[test]
fn unrelated_files_oversized_inputs_and_links_are_refused() {
    let dir = dir();
    let path = dir.path().join("不能覆盖.smeffect.json");
    let bytes = Document::new("工程").unwrap().encode().unwrap();
    fs::write(&path, &bytes).unwrap();
    assert!(EffectTemplateFileStore::select(&path).is_err());
    assert!(EffectTemplateFileStore::read(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(EffectTemplateFileStore::select(&dir.path().join("错误.json")).is_err());
    let large = dir.path().join("太大.smeffect.json");
    fs::File::create(&large)
        .unwrap()
        .set_len((MAX_EFFECT_TEMPLATE_BYTES + 1) as u64)
        .unwrap();
    assert!(EffectTemplateFileStore::read(&large).is_err());
    assert!(EffectTemplateFileStore::read(dir.path()).is_err());
    #[cfg(unix)]
    {
        let link = dir.path().join("链接.smeffect.json");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(EffectTemplateFileStore::select(&link).is_err());
        assert!(EffectTemplateFileStore::read(&link).is_err());
    }
}
