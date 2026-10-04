//! Private, non-blocking ownership of one background management transaction.
use std::{
    fs::{File, OpenOptions, TryLockError},
    path::Path,
};

#[derive(Debug)]
pub(super) struct ManagerLock(File);
impl ManagerLock {
    pub(super) fn acquire(root: &Path) -> Result<Self, String> {
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(root.join("manager.lock"))
            .map_err(|e| e.to_string())?;
        file.try_lock().map_err(lock_error)?;
        Ok(Self(file))
    }
}
impl Drop for ManagerLock {
    fn drop(&mut self) {
        // Close alone may leave a cloned/inherited descriptor holding this transaction.
        // Only a successfully acquired guard explicitly unlocks, including early exits.
        let _ = self.0.unlock();
    }
}
fn lock_error(error: TryLockError) -> String {
    match error {
        TryLockError::WouldBlock => "另一个应用正在管理此后台".into(),
        TryLockError::Error(error) => format!("无法取得后台管理锁：{error}"),
    }
}

#[cfg(test)]
#[path = "manager_lock_tests.rs"]
mod tests;
