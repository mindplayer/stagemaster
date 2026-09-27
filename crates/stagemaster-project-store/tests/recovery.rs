use stagemaster_project::{Document, EditCommand};
use stagemaster_project_store::{DiskFile, MAX_RECOVERY_RECORDS, RecoveryState, RecoveryStore};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn directory() -> tempfile::TempDir {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}
fn path(store: &RecoveryStore, id: &str) -> PathBuf {
    store.root().join(format!("{id}.recovery.json"))
}
fn rename(doc: &mut Document, name: &str) {
    doc.edit(EditCommand::SetInfo {
        name: name.into(),
        description: String::new(),
    })
    .unwrap();
}
#[test]
fn checkpoints_do_not_touch_source_or_revision_and_active_sessions_are_protected() {
    let dir = directory();
    let source = dir.path().join("original.json");
    let doc = DiskFile::select(&source)
        .unwrap()
        .save(&Document::new("原工程").unwrap())
        .unwrap()
        .document;
    let bytes = fs::read(&source).unwrap();
    let store = RecoveryStore::open(&dir.path().join("recovery")).unwrap();
    let mut session = store.begin().unwrap();
    session
        .checkpoint(&doc, Some(source.to_string_lossy().into()), 100)
        .unwrap();
    let entry = store.list(200).unwrap().entries.remove(0);
    assert_eq!(entry.state, RecoveryState::Active);
    assert!(!entry.can_discard);
    assert!(store.claim(&entry.id, &entry.token).is_err());
    assert!(store.discard(&entry.id, &entry.token).is_err());
    drop(session);
    let entry = store.list(200).unwrap().entries.remove(0);
    assert_eq!(entry.state, RecoveryState::Ready);
    let mut claim = store.claim(&entry.id, &entry.token).unwrap();
    assert_eq!(claim.document, doc);
    assert_eq!(claim.source_file.as_deref(), source.to_str());
    assert_eq!(
        store.list(200).unwrap().entries[0].state,
        RecoveryState::Active
    );
    assert!(store.claim(&entry.id, &entry.token).is_err());
    claim.discard().unwrap();
    claim.discard().unwrap();
    assert!(store.list(200).unwrap().entries.is_empty());
    assert_eq!(fs::read(source).unwrap(), bytes);
}
#[test]
fn cancelling_claim_preserves_record_and_stale_tokens_cannot_delete_new_checkpoints() {
    let dir = directory();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let mut doc = Document::new("一").unwrap();
    let mut session = store.begin().unwrap();
    session.checkpoint(&doc, None, 1).unwrap();
    let old = store.list(1).unwrap().entries.remove(0);
    rename(&mut doc, "二");
    session.checkpoint(&doc, None, 2).unwrap();
    drop(session);
    assert!(store.claim(&old.id, &old.token).is_err());
    assert!(store.discard(&old.id, &old.token).is_err());
    let entry = store.list(3).unwrap().entries.remove(0);
    drop(store.claim(&entry.id, &entry.token).unwrap());
    assert!(path(&store, &entry.id).exists());
    let restored = store.claim(&entry.id, &entry.token).unwrap();
    assert_eq!(restored.document, doc);
    drop(restored);
    store.discard(&entry.id, &entry.token).unwrap();
    assert!(store.list(3).unwrap().entries.is_empty());
}
#[test]
fn damaged_unsupported_old_and_unknown_files_are_not_silently_removed() {
    let dir = directory();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let mut session = store.begin().unwrap();
    let id = session.id().to_owned();
    session
        .checkpoint(&Document::new("较早工程").unwrap(), None, 1)
        .unwrap();
    drop(session);
    let entry = store
        .list(31 * 24 * 60 * 60 * 1000)
        .unwrap()
        .entries
        .remove(0);
    assert!(entry.older);
    assert_eq!(entry.state, RecoveryState::Ready);
    drop(store.claim(&entry.id, &entry.token).unwrap());
    let original = fs::read(path(&store, &id)).unwrap();
    for data in [
        b"broken".to_vec(),
        String::from_utf8(original.clone())
            .unwrap()
            .replacen("\"version\":1", "\"version\":999", 1)
            .into_bytes(),
        String::from_utf8(original.clone())
            .unwrap()
            .replacen("较早工程", "摘要被改", 1)
            .into_bytes(),
    ] {
        fs::write(path(&store, &id), &data).unwrap();
        let entry = store.list(100).unwrap().entries.remove(0);
        assert_eq!(entry.state, RecoveryState::Damaged);
        assert!(entry.can_discard);
        assert!(entry.problem.is_some());
        assert!(store.claim(&entry.id, &entry.token).is_err());
        assert_eq!(fs::read(path(&store, &id)).unwrap(), data);
    }
    fs::write(store.root().join("do-not-delete.txt"), b"external").unwrap();
    let entry = store.list(100).unwrap().entries.remove(0);
    store.discard(&entry.id, &entry.token).unwrap();
    assert_eq!(
        fs::read(store.root().join("do-not-delete.txt")).unwrap(),
        b"external"
    );
    assert!(store.discard("../escape", "x").is_err());
}
#[test]
fn clear_keeps_live_lease_and_capacity_never_evicts_existing_work() {
    let dir = directory();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let doc = Document::new("未保存").unwrap();
    for _ in 0..MAX_RECOVERY_RECORDS {
        let mut s = store.begin().unwrap();
        s.checkpoint(&doc, None, 1).unwrap();
    }
    let mut extra = store.begin().unwrap();
    assert!(extra.checkpoint(&doc, None, 2).is_err());
    assert_eq!(store.list(3).unwrap().entries.len(), MAX_RECOVERY_RECORDS);
    let old = store.list(3).unwrap().entries.remove(0);
    store.discard(&old.id, &old.token).unwrap();
    extra.checkpoint(&doc, None, 3).unwrap();
    extra.clear().unwrap();
    extra.clear().unwrap();
    // Listing may collect only abandoned, empty leases; this live lease must survive.
    store.list(3).unwrap();
    assert!(store.root().join(format!("{}.lease", extra.id())).exists());
    extra.checkpoint(&doc, None, 4).unwrap();
    assert_eq!(
        store
            .list(4)
            .unwrap()
            .entries
            .iter()
            .filter(|e| e.state == RecoveryState::Active)
            .count(),
        1
    );
}
#[cfg(unix)]
#[test]
fn failed_atomic_write_preserves_last_checkpoint_and_retry_succeeds() {
    use std::os::unix::fs::PermissionsExt;
    let dir = directory();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let mut session = store.begin().unwrap();
    let mut doc = Document::new("上次检查点").unwrap();
    session.checkpoint(&doc, None, 1).unwrap();
    let before = fs::read(path(&store, session.id())).unwrap();
    rename(&mut doc, "下一检查点");
    fs::set_permissions(store.root(), fs::Permissions::from_mode(0o500)).unwrap();
    let result = session.checkpoint(&doc, None, 2);
    fs::set_permissions(store.root(), fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        result.is_err(),
        "run this test as the normal development user, not root"
    );
    assert_eq!(fs::read(path(&store, session.id())).unwrap(), before);
    session.checkpoint(&doc, None, 3).unwrap();
    assert_ne!(fs::read(path(&store, session.id())).unwrap(), before);
}
#[cfg(unix)]
#[test]
fn links_and_nonregular_records_are_rejected_without_touching_their_targets() {
    let dir = directory();
    let root = dir.path().join("recovery");
    let store = RecoveryStore::open(&root).unwrap();
    let external = dir.path().join("external");
    fs::write(&external, b"preserve").unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    std::os::unix::fs::symlink(&external, path(&store, &id)).unwrap();
    let entry = store.list(1).unwrap().entries.remove(0);
    assert!(!entry.can_discard);
    assert_eq!(entry.state, RecoveryState::Damaged);
    assert!(store.discard(&entry.id, &entry.token).is_err());
    assert!(store.claim(&entry.id, &entry.token).is_err());
    assert_eq!(fs::read(&external).unwrap(), b"preserve");
    let rootlink = dir.path().join("root-link");
    std::os::unix::fs::symlink(root, &rootlink).unwrap();
    assert!(RecoveryStore::open(&rootlink).is_err());
}
#[test]
fn recovery_process_helper() {
    let Some(root) = std::env::var_os("STAGEMASTER_RECOVERY_CHILD") else {
        return;
    };
    let store = RecoveryStore::open(Path::new(&root)).unwrap();
    let mut session = store.begin().unwrap();
    session
        .checkpoint(&Document::new("进程中断恢复").unwrap(), None, 10)
        .unwrap();
    println!("READY:{}", session.id());
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
#[test]
fn os_lease_is_released_after_an_actual_process_kill() {
    let dir = directory();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "recovery_process_helper", "--nocapture"])
        .env("STAGEMASTER_RECOVERY_CHILD", dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert!(output.read_line(&mut line).unwrap() > 0);
        if line.starts_with("READY:") {
            break;
        }
    }
    let id = line.trim().strip_prefix("READY:").unwrap().to_owned();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let occupied = store.list(11).unwrap().entries.remove(0);
    assert_eq!(occupied.id, id);
    assert_eq!(occupied.state, RecoveryState::Active);
    assert!(store.discard(&occupied.id, &occupied.token).is_err());
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
    let orphan = store.root().join(format!(".recovery-{id}-abandoned.tmp"));
    fs::write(&orphan, b"partial").unwrap();
    let ready = store.list(12).unwrap().entries.remove(0);
    assert_eq!(ready.state, RecoveryState::Ready);
    let mut restored = store.claim(&ready.id, &ready.token).unwrap();
    assert_eq!(restored.document.view().name, "进程中断恢复");
    restored.discard().unwrap();
    assert!(!orphan.exists());
    assert!(store.list(12).unwrap().entries.is_empty());
}

#[test]
fn orphan_cleanup_never_touches_an_active_empty_session_or_unknown_files() {
    let dir = directory();
    let store = RecoveryStore::open(dir.path()).unwrap();
    let active = store.begin().unwrap();
    let abandoned = uuid::Uuid::new_v4().to_string();
    let lease = store.root().join(format!("{abandoned}.lease"));
    fs::write(&lease, b"").unwrap();
    let incomplete = store
        .root()
        .join(format!(".recovery-{abandoned}-partial.tmp"));
    fs::write(&incomplete, b"partial").unwrap();
    let other = store.root().join("foreign.tmp");
    fs::write(&other, b"preserve").unwrap();
    assert!(store.list(1).unwrap().entries.is_empty());
    assert!(!lease.exists());
    assert!(!incomplete.exists());
    assert!(other.exists());
    assert!(store.root().join(format!("{}.lease", active.id())).exists());
}
