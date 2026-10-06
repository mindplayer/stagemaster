use super::*;

fn directory() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
fn populate(store: &RecoveryStore, count: u128) {
    for value in 1..=count {
        let id = Uuid::from_u128(value).to_string();
        fs::write(store.path(&id), b"{damaged record").unwrap();
    }
}

#[test]
fn overfull_directory_does_not_materialize_all_candidate_ids() {
    let directory = directory();
    let store = RecoveryStore::open(directory.path()).unwrap();
    populate(&store, 512);
    let ids = store.ids().unwrap();
    eprintln!(
        "real_directory_records=512 candidate_ids={} limit={MAX_RECOVERY_RECORDS}",
        ids.len()
    );
    assert_eq!(ids.len(), MAX_RECOVERY_RECORDS);
    let expected: Vec<_> = (1..=64)
        .map(|value| Uuid::from_u128(value).to_string())
        .collect();
    assert_eq!(ids, expected);
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 512);
}

#[test]
fn overfull_catalog_preserves_lexical_selection_omitted_count_and_unknown_files() {
    let directory = directory();
    let store = RecoveryStore::open(directory.path()).unwrap();
    populate(&store, 130);
    let unknown = store.root().join("未知资料.json");
    fs::write(&unknown, b"not ours").unwrap();
    fs::write(
        store
            .root()
            .join("FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF.recovery.json"),
        b"invalid canonical ID",
    )
    .unwrap();
    let catalog = store.list(200).unwrap();
    assert_eq!(catalog.entries.len(), MAX_RECOVERY_RECORDS);
    assert_eq!(catalog.omitted, 66);
    let selected: Vec<_> = catalog
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    let expected: Vec<_> = (1..=64)
        .map(|value| Uuid::from_u128(value).to_string())
        .collect();
    assert_eq!(selected, expected);
    assert!(
        catalog
            .entries
            .iter()
            .all(|entry| entry.state == RecoveryState::Damaged)
    );
    for value in 1..=130 {
        assert_eq!(
            fs::read(store.path(&Uuid::from_u128(value).to_string())).unwrap(),
            b"{damaged record"
        );
    }
    assert_eq!(fs::read(unknown).unwrap(), b"not ours");
}

#[test]
fn a_full_catalog_refuses_new_snapshots_but_keeps_existing_updates() {
    let directory = directory();
    let store = RecoveryStore::open(directory.path()).unwrap();
    let mut session = store.begin().unwrap();
    let mut document = Document::new("旧恢复点").unwrap();
    session.checkpoint(&document, None, 1).unwrap();
    populate(&store, 63);
    let before = fs::read(store.path(session.id())).unwrap();
    let mut another = store.begin().unwrap();
    assert!(
        another
            .checkpoint(&document, None, 2)
            .unwrap_err()
            .contains("64 份")
    );
    assert_eq!(fs::read(store.path(session.id())).unwrap(), before);
    assert!(!store.path(another.id()).exists());
    document
        .edit(stagemaster_project::EditCommand::SetInfo {
            name: "原会话更新".into(),
            description: String::new(),
        })
        .unwrap();
    session.checkpoint(&document, None, 3).unwrap();
    let catalog = store.list(4).unwrap();
    assert_eq!(catalog.entries.len(), 64);
    assert_eq!(catalog.omitted, 0);
    let current = catalog
        .entries
        .iter()
        .find(|entry| entry.id == session.id())
        .unwrap();
    assert_eq!(current.project_name.as_deref(), Some("原会话更新"));
    assert_eq!(current.state, RecoveryState::Active);
    assert!(!current.can_discard);
    assert!(store.claim(&current.id, &current.token).is_err());
    assert!(store.discard(&current.id, &current.token).is_err());
    for value in 1..=63 {
        assert_eq!(
            fs::read(store.path(&Uuid::from_u128(value).to_string())).unwrap(),
            b"{damaged record"
        );
    }
}

#[cfg(unix)]
#[test]
fn non_regular_record_names_count_without_following_or_discarding_their_targets() {
    use std::os::unix::fs::symlink;
    let directory = directory();
    let store = RecoveryStore::open(directory.path()).unwrap();
    populate(&store, 80);
    let folder_id = Uuid::from_u128(2).to_string();
    let link_id = Uuid::from_u128(3).to_string();
    fs::remove_file(store.path(&folder_id)).unwrap();
    fs::create_dir(store.path(&folder_id)).unwrap();
    let target = store.root().join("unknown.txt");
    fs::write(&target, b"keep target").unwrap();
    fs::remove_file(store.path(&link_id)).unwrap();
    symlink(&target, store.path(&link_id)).unwrap();
    let catalog = store.list(1).unwrap();
    assert_eq!(catalog.omitted, 16);
    for id in [folder_id, link_id] {
        let entry = catalog.entries.iter().find(|entry| entry.id == id).unwrap();
        assert_eq!(entry.state, RecoveryState::Damaged);
        assert!(!entry.can_discard);
        assert!(store.discard(&entry.id, &entry.token).is_err());
    }
    assert_eq!(fs::read(&target).unwrap(), b"keep target");
    assert!(
        fs::symlink_metadata(store.path(&Uuid::from_u128(3).to_string()))
            .unwrap()
            .is_symlink()
    );
}
