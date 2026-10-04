use super::*;
use crate::recent::store::Store;
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

fn directory() -> tempfile::TempDir {
    tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
}

#[test]
fn released_transaction_is_not_retained_by_a_live_child_with_its_descriptor() {
    let dir = directory();
    let store = Store(dir.path().into());
    store.remember(&dir.path().join("a.json"), "甲").unwrap();
    let lock = WriteLock::acquire(dir.path()).unwrap();
    let mut child = Holder::start(dir.path(), lock.0.try_clone().unwrap());
    assert!(child.child.try_wait().unwrap().is_none());
    let probe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.path().join("recent.lock"))
        .unwrap();
    let error = probe.try_lock().unwrap_err();
    println!(
        "live holder PID {}, raw lock error {error:?}",
        child.child.id()
    );
    assert!(matches!(error, TryLockError::WouldBlock));
    assert!(store.remember(&dir.path().join("b.json"), "乙").is_err());
    assert_eq!(store.list().unwrap().len(), 1);
    drop(lock);
    let result = store.remember(&dir.path().join("b.json"), "乙");
    println!("next real catalog transaction while inherited descriptor remains: {result:?}");
    assert!(child.child.try_wait().unwrap().is_none());
    child.finish();
    result.unwrap();
    assert_eq!(store.list().unwrap().len(), 2);
}

#[test]
fn old_duplicate_close_cannot_unlock_the_next_owner() {
    let dir = directory();
    let first = WriteLock::acquire(dir.path()).unwrap();
    let duplicate = first.0.try_clone().unwrap();
    drop(first);
    let next = WriteLock::acquire(dir.path()).unwrap();
    drop(duplicate);
    assert_eq!(
        WriteLock::acquire(dir.path()).unwrap_err(),
        "其他窗口正在更新最近工程，请稍后重试"
    );
    drop(next);
    WriteLock::acquire(dir.path()).unwrap();
}

#[test]
fn early_error_and_unwind_release_with_an_open_duplicate() {
    let dir = directory();
    for unwind in [false, true] {
        let mut duplicate = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let lock = WriteLock::acquire(dir.path()).unwrap();
            duplicate = Some(lock.0.try_clone().unwrap());
            assert!(!unwind, "controlled transaction unwind");
            Err::<(), _>("controlled transaction early error")
        }));
        if unwind {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), Err("controlled transaction early error"));
        }
        let store = Store(dir.path().into());
        store
            .remember(&dir.path().join("a.json"), "仍可保存")
            .unwrap();
        drop(duplicate);
    }
}

#[test]
fn rejected_acquire_never_unlocks_the_real_writer_or_changes_catalog() {
    let dir = directory();
    let store = Store(dir.path().into());
    store
        .remember(&dir.path().join("a.json"), "原工程")
        .unwrap();
    let before = fs::read(dir.path().join("recent.json")).unwrap();
    let lock = WriteLock::acquire(dir.path()).unwrap();
    for _ in 0..3 {
        assert!(
            store
                .remember(&dir.path().join("b.json"), "其他写者")
                .is_err()
        );
        assert_eq!(fs::read(dir.path().join("recent.json")).unwrap(), before);
    }
    assert!(WriteLock::acquire(dir.path()).is_err());
    drop(lock);
    store
        .remember(&dir.path().join("b.json"), "取得后保存")
        .unwrap();
}

#[test]
fn non_file_lock_and_system_error_are_not_reported_as_a_busy_writer() {
    let dir = directory();
    fs::create_dir(dir.path().join("recent.lock")).unwrap();
    assert_eq!(
        WriteLock::acquire(dir.path()).unwrap_err(),
        "最近工程锁文件无效"
    );
    let message = lock_error(TryLockError::Error(std::io::Error::from_raw_os_error(13)));
    assert!(message.starts_with("无法取得最近工程写锁："));
    assert!(message.contains("13"));
}

struct Holder {
    child: Child,
    directory: PathBuf,
    finished: bool,
}
impl Holder {
    fn start(directory: &Path, inherited: File) -> Self {
        let log = File::create(directory.join("child.log")).unwrap();
        let mut holder = Self {
            child: Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "recent::store::write_lock::tests::inherited_file_holder",
                    "--ignored",
                    "--nocapture",
                ])
                .env("PROJECT003_LOCK_DIRECTORY", directory)
                .stdin(Stdio::from(inherited))
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap(),
            directory: directory.into(),
            finished: false,
        };
        let end = Instant::now() + Duration::from_secs(6);
        while !directory.join("child-ready").exists() {
            assert!(
                holder.child.try_wait().unwrap().is_none(),
                "child exited early"
            );
            assert!(Instant::now() < end, "child ready deadline");
            std::thread::sleep(Duration::from_millis(5));
        }
        holder
    }
    fn finish(&mut self) {
        fs::write(self.directory.join("child-release"), "release").unwrap();
        let status = self.child.wait().unwrap();
        self.finished = true;
        assert!(
            status.success(),
            "{}",
            fs::read_to_string(self.directory.join("child.log")).unwrap()
        );
    }
}
impl Drop for Holder {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

// Invoked by the parent test with an inherited catalog descriptor on standard input.
#[test]
#[ignore = "subprocess entry; exercised by released_transaction_is_not_retained_by_a_live_child_with_its_descriptor"]
fn inherited_file_holder() {
    let dir = PathBuf::from(std::env::var_os("PROJECT003_LOCK_DIRECTORY").unwrap());
    fs::write(dir.join("child-ready"), "ready").unwrap();
    let end = Instant::now() + Duration::from_secs(6);
    while !dir.join("child-release").exists() {
        assert!(Instant::now() < end, "parent release deadline");
        std::thread::sleep(Duration::from_millis(5));
    }
}
