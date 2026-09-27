#[path = "../../stagemaster-project/tests/support/capacity.rs"]
mod support;
use stagemaster_project::{Document, EditCommand, MAX_BYTES};
use stagemaster_project_store::{DiskFile, RecoveryState, RecoveryStore};
use std::{fs, path::PathBuf};

fn directory() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}

#[test]
fn oversized_indentation_does_not_block_save_reopen_or_recovery_as_a_copy() {
    let dir = directory();
    let source = dir.path().join("source.json");
    let root = support::root_with_scenes(6000);
    assert!(serde_json::to_vec_pretty(&root).unwrap().len() > MAX_BYTES);
    let original = serde_json::to_vec(&root).unwrap();
    fs::write(&source, &original).unwrap();
    let (mut doc, mut file) = DiskFile::open(&source).unwrap();
    doc.edit(EditCommand::SetInfo {
        name: "容量验收".into(),
        description: "首次编辑保存".into(),
    })
    .unwrap();
    let saved = file.save(&doc).unwrap().document;
    assert!(saved.same_content(&doc));
    assert_eq!(DiskFile::open(&source).unwrap().0, saved);
    let source_bytes = fs::read(&source).unwrap();
    assert!(source_bytes.len() <= MAX_BYTES);
    assert_ne!(source_bytes, original);

    let mut draft = saved;
    draft
        .edit(EditCommand::SetInfo {
            name: "未保存大工程".into(),
            description: "需要恢复的未保存修改".into(),
        })
        .unwrap();
    let store = RecoveryStore::open(&dir.path().join("recovery")).unwrap();
    let mut session = store.begin().unwrap();
    session
        .checkpoint(&draft, Some(source.to_string_lossy().into()), 100)
        .unwrap();
    drop(session);
    let entry = store.list(200).unwrap().entries.remove(0);
    assert_eq!(entry.state, RecoveryState::Ready);
    let mut claim = store.claim(&entry.id, &entry.token).unwrap();
    assert_eq!(claim.document, draft);
    let copied = dir.path().join("recovered.json");
    let receipt = DiskFile::select(&copied)
        .unwrap()
        .save(&claim.document)
        .unwrap();
    assert!(receipt.document.same_content(&draft));
    assert_eq!(DiskFile::open(&copied).unwrap().0, receipt.document);
    claim.discard().unwrap();
    assert!(store.list(300).unwrap().entries.is_empty());
    assert_eq!(fs::read(&source).unwrap(), source_bytes);
}

#[test]
fn full_capacity_snapshot_and_revision_save_fit_and_rejected_edits_preserve_both() {
    let dir = directory();
    let root = support::root_of_size(MAX_BYTES - 38);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let path = dir.path().join("full.json");
    let mut file = DiskFile::select(&path).unwrap();
    doc = file.save(&doc).unwrap().document;
    assert_eq!(fs::metadata(&path).unwrap().len(), MAX_BYTES as u64);
    let store = RecoveryStore::open(&dir.path().join("recovery")).unwrap();
    let mut session = store.begin().unwrap();
    session.checkpoint(&doc, None, 100).unwrap();
    let entry = store.list(100).unwrap().entries.remove(0);
    let before = fs::read(&path).unwrap();
    assert!(
        doc.edit(EditCommand::SetInfo {
            name: "容量验收".into(),
            description: "x".into(),
        })
        .unwrap_err()
        .contains("8 MiB")
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(store.list(200).unwrap().entries[0].token, entry.token);
    drop(session);
    let candidate = store.claim(&entry.id, &entry.token).unwrap();
    assert_eq!(candidate.document, doc);
    let second = file.save(&candidate.document).unwrap().document;
    assert_eq!(fs::metadata(&path).unwrap().len(), MAX_BYTES as u64);
    assert!(DiskFile::open(&path).unwrap().0.same_content(&second));
}
