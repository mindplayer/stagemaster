use super::*;
use crate::execution::{
    files,
    manager::Manager,
    manager::tests::{binary, document, runtime},
};
use stagemaster_execution_client::Selection;
use std::{fs, path::PathBuf};

#[path = "manager_lock_test_holder.rs"]
mod holder;

fn directory() -> tempfile::TempDir {
    let directory =
        tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    directory
}

#[test]
fn released_manager_transaction_does_not_remain_owned_by_a_live_child() {
    let dir = directory();
    let mut manager = Manager::new(dir.path().into(), binary());
    let lock = manager.reserve_editor_audio().unwrap();
    let mut child = holder::Holder::start(dir.path(), lock.0.try_clone().unwrap());
    let probe = OpenOptions::new()
        .write(true)
        .open(dir.path().join("manager.lock"))
        .unwrap();
    let error = probe.try_lock().unwrap_err();
    println!(
        "live holder PID {}, raw manager lock error {error:?}",
        child.id()
    );
    assert!(matches!(error, TryLockError::WouldBlock));
    assert_eq!(
        manager.reserve_editor_audio().unwrap_err(),
        "另一个应用正在管理此后台"
    );
    drop(lock);
    let result = manager.reserve_editor_audio();
    println!("next real editor reservation while inherited descriptor remains: {result:?}");
    assert!(child.running());
    child.finish();
    let next = result.unwrap();
    assert!(!dir.path().join("current").exists());
    drop(next);
    manager.reserve_editor_audio().unwrap();
}

#[test]
fn old_duplicate_close_does_not_unlock_the_next_management_owner() {
    let dir = directory();
    let mut manager = Manager::new(dir.path().into(), binary());
    let first = manager.reserve_editor_audio().unwrap();
    let duplicate = first.0.try_clone().unwrap();
    drop(first);
    let next = manager.reserve_editor_audio().unwrap();
    drop(duplicate);
    assert_eq!(
        manager.reserve_editor_audio().unwrap_err(),
        "另一个应用正在管理此后台"
    );
    drop(next);
    manager.reserve_editor_audio().unwrap();
}

#[test]
fn early_error_and_unwind_release_management_lock_with_an_open_duplicate() {
    let dir = directory();
    for unwind in [false, true] {
        let mut duplicate = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let lock = files::lock(dir.path()).unwrap();
            duplicate = Some(lock.0.try_clone().unwrap());
            assert!(!unwind, "controlled management unwind");
            Err::<(), _>("controlled management early error")
        }));
        if unwind {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), Err("controlled management early error"));
        }
        let mut manager = Manager::new(dir.path().into(), binary());
        manager.reserve_editor_audio().unwrap();
        drop(duplicate);
    }
}

#[test]
fn rejected_acquire_does_not_release_real_owner_or_publish_a_background_record() {
    runtime().block_on(async {
        let dir = directory();
        let owner = files::lock(dir.path()).unwrap();
        let mut manager = Manager::new(dir.path().into(), binary());
        for _ in 0..3 {
            let error = manager
                .prepare(document(), vec![Selection::AudioTimeline {}], None)
                .await
                .err()
                .unwrap();
            assert_eq!(error, "另一个应用正在管理此后台");
            assert_eq!(manager.reserve_editor_audio().unwrap_err(), error);
            assert!(!dir.path().join("current").exists());
        }
        assert!(files::lock(dir.path()).is_err());
        drop(owner);
        let error = manager
            .prepare(document(), vec![Selection::AudioTimeline {}], None)
            .await
            .err()
            .unwrap();
        assert!(error.contains("音乐"), "{error}");
        assert!(!dir.path().join("current").exists());
        manager.reserve_editor_audio().unwrap();
    });
}

#[test]
fn actual_busy_and_system_errors_are_distinct_and_open_failure_is_not_busy() {
    assert_eq!(
        lock_error(TryLockError::WouldBlock),
        "另一个应用正在管理此后台"
    );
    let system = lock_error(TryLockError::Error(std::io::Error::from_raw_os_error(13)));
    assert!(system.starts_with("无法取得后台管理锁："), "{system}");
    assert!(system.contains("13"));
    let dir = directory();
    fs::create_dir(dir.path().join("manager.lock")).unwrap();
    let error = files::lock(dir.path()).unwrap_err();
    assert_ne!(error, "另一个应用正在管理此后台");
    assert!(dir.path().join("manager.lock").is_dir());
}
