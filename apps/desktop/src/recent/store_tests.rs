use super::*;
fn directory() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}
#[test]
fn recent_entries_are_bounded_deduplicated_and_survive_reopen() {
    let dir = directory();
    let store = Store(dir.path().join("catalog"));
    assert!(store.list().unwrap().is_empty());
    for i in 0..15 {
        store
            .remember(&dir.path().join(format!("{i}.json")), &format!("工程{i}"))
            .unwrap();
    }
    let entries = store.list().unwrap();
    assert_eq!(entries.len(), 12);
    assert_eq!(entries[0].name, "工程14");
    let id = entries[5].id.clone();
    let path = entries[5].path.clone();
    store.remember(&path, "新的名称").unwrap();
    let reopened = Store(store.0.clone()).list().unwrap();
    assert_eq!(reopened.len(), 12);
    assert_eq!(reopened[0].id, id);
    assert_eq!(reopened[0].name, "新的名称");
    assert_eq!(store.resolve(&id).unwrap(), path);
}

#[test]
fn forgetting_only_removes_metadata_and_unknown_ids_cannot_open_paths() {
    let dir = directory();
    let path = dir.path().join("工程.json");
    fs::write(&path, "原工程内容").unwrap();
    let store = Store(dir.path().join("catalog"));
    store.remember(&path, "工程").unwrap();
    let id = store.list().unwrap()[0].id.clone();
    assert!(store.resolve(path.to_str().unwrap()).is_err());
    assert!(store.resolve("../工程.json").is_err());
    store.forget(&id).unwrap();
    assert!(store.resolve(&id).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "原工程内容");
}

#[test]
fn corrupt_or_oversized_catalog_is_reported_without_overwriting_it() {
    let dir = directory();
    let store = Store(dir.path().to_owned());
    let path = dir.path().join("recent.json");
    for data in [
        b"{bad".to_vec(),
        vec![b' '; usize::try_from(MAX_BYTES + 1).unwrap()],
    ] {
        fs::write(&path, &data).unwrap();
        assert!(
            store
                .remember(&dir.path().join("show.json"), "工程")
                .is_err()
        );
        assert!(store.forget("unknown").is_err());
        assert_eq!(fs::read(&path).unwrap(), data);
    }
}

#[test]
fn catalog_validation_rejects_version_duplicates_and_relative_paths() {
    let dir = directory();
    let store = Store(dir.path().to_owned());
    let entry = serde_json::json!({"id":uuid::Uuid::new_v4().to_string(),"name":"工程","path":dir.path().join("a.json"),"openedAtMs":1});
    let mut relative = entry.clone();
    relative["path"] = "relative.json".into();
    for value in [
        serde_json::json!({"version":2,"entries":[]}),
        serde_json::json!({"version":1,"entries":[entry.clone(),entry.clone()]}),
        serde_json::json!({"version":1,"entries":[relative]}),
    ] {
        fs::write(
            dir.path().join("recent.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert!(store.list().is_err());
    }
}

#[test]
fn multiple_instances_merge_latest_data_and_busy_writer_preserves_it() {
    let dir = directory();
    let first = Store(dir.path().to_owned());
    let second = Store(dir.path().to_owned());
    first.remember(&dir.path().join("a.json"), "甲").unwrap();
    second.remember(&dir.path().join("b.json"), "乙").unwrap();
    assert_eq!(first.list().unwrap().len(), 2);
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.path().join("recent.lock"))
        .unwrap();
    lock.lock().unwrap();
    assert!(second.remember(&dir.path().join("c.json"), "丙").is_err());
    assert_eq!(first.list().unwrap().len(), 2);
    drop(lock);
    second.remember(&dir.path().join("c.json"), "丙").unwrap();
    assert_eq!(first.list().unwrap().len(), 3);
}

#[test]
fn invalid_new_record_leaves_previous_catalog_untouched() {
    let dir = directory();
    let store = Store(dir.path().to_owned());
    store
        .remember(&dir.path().join("a.json"), "原工程")
        .unwrap();
    let original = fs::read(dir.path().join("recent.json")).unwrap();
    assert!(store.remember(Path::new("a.json"), "相对路径").is_err());
    assert!(
        store
            .remember(&dir.path().join("b.json"), &"a".repeat(513))
            .is_err()
    );
    assert_eq!(fs::read(dir.path().join("recent.json")).unwrap(), original);
}
