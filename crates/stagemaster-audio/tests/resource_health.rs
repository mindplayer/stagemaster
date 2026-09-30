use stagemaster_audio::{MAX_FILE_BYTES, ResourceFileHealth as Health, ResourceSource, Resources};
use std::{fs, sync::atomic::AtomicBool};
#[test]
fn inspection_distinguishes_local_and_portable_copies_without_repairing_them() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("original.mp3");
    fs::write(
        &source,
        b"integrity fixture; no audio decoding in inspection",
    )
    .unwrap();
    let resources = Resources::new(temp.path().join("cache"));
    let cancel = AtomicBool::new(false);
    let (digest, cache) = resources.import(&source, "mp3", None, &cancel).unwrap();
    let unsaved = resources.inspect(&digest, "mp3", None, &cancel).unwrap();
    assert_eq!(unsaved.local, Health::Valid);
    assert_eq!(unsaved.companion, Health::NotSaved);
    let project = temp.path().join("节目.json");
    let absent = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert_eq!(absent.local_source, Some(ResourceSource::Cache));
    assert_eq!(absent.companion, Health::Missing);
    assert!(!temp.path().join("节目.json.assets").exists());
    resources.archive(&digest, "mp3", None, &project).unwrap();
    let companion = temp
        .path()
        .join("节目.json.assets")
        .join(format!("{digest}.mp3"));
    let original = fs::read(&companion).unwrap();
    let ready = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert_eq!(ready.local, Health::Valid);
    assert_eq!(ready.companion, Health::Valid);
    fs::write(&cache, b"corrupted cache").unwrap();
    let broken_cache = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert!(matches!(broken_cache.local, Health::Invalid { .. }));
    assert_eq!(broken_cache.local_source, Some(ResourceSource::Cache));
    assert_eq!(broken_cache.companion, Health::Valid);
    assert_eq!(fs::read(&cache).unwrap(), b"corrupted cache");
    assert_eq!(fs::read(&companion).unwrap(), original);
    fs::remove_file(&cache).unwrap();
    let portable = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert_eq!(portable.local_source, Some(ResourceSource::Companion));
    assert_eq!(portable.local, Health::Valid);
    fs::write(&companion, b"corrupted companion").unwrap();
    let broken = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert!(matches!(broken.companion, Health::Invalid { .. }));
    assert_eq!(broken.local, broken.companion);
    fs::remove_file(&companion).unwrap();
    let missing = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert_eq!(missing.local, Health::Missing);
    assert_eq!(missing.companion, Health::Missing);
    assert_eq!(missing.local_source, None);
    assert!(!cache.exists());
}
#[test]
fn invalid_keys_cancellation_and_non_file_or_oversized_companions_are_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let resources = Resources::new(temp.path().join("absent-cache"));
    let project = temp.path().join("show.json");
    let cancel = AtomicBool::new(false);
    let digest = "ab".repeat(32);
    assert!(
        resources
            .inspect("../unsafe", "mp3", Some(&project), &cancel)
            .is_err()
    );
    assert!(
        resources
            .inspect(&digest, "../../unsafe", Some(&project), &cancel)
            .is_err()
    );
    assert!(
        resources
            .inspect(&digest, "mp3", None, &AtomicBool::new(true))
            .unwrap_err()
            .contains("取消")
    );
    assert!(!temp.path().join("absent-cache").exists());
    let path = temp
        .path()
        .join("show.json.assets")
        .join(format!("{digest}.mp3"));
    fs::create_dir_all(&path).unwrap();
    assert!(matches!(
        resources
            .inspect(&digest, "mp3", Some(&project), &cancel)
            .unwrap()
            .companion,
        Health::Invalid { .. }
    ));
    fs::remove_dir(&path).unwrap();
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_FILE_BYTES + 1).unwrap();
    let result = resources
        .inspect(&digest, "mp3", Some(&project), &cancel)
        .unwrap();
    assert!(matches!(result.companion, Health::Invalid { message } if message.contains("512 MiB")));
    assert_eq!(file.metadata().unwrap().len(), MAX_FILE_BYTES + 1);
}
