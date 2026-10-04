//! Private, non-blocking ownership of one catalog transaction's lock.
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    path::Path,
};

#[derive(Debug)]
pub(super) struct WriteLock(File);
impl WriteLock {
    pub(super) fn acquire(directory: &Path) -> Result<Self, String> {
        let path = directory.join("recent.lock");
        if fs::symlink_metadata(&path).is_ok_and(|m| !m.is_file()) {
            return Err("最近工程锁文件无效".into());
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|error| format!("无法打开最近工程写锁：{error}"))?;
        file.try_lock().map_err(lock_error)?;
        Ok(Self(file))
    }
}
impl Drop for WriteLock {
    fn drop(&mut self) {
        // Closing only our descriptor can retain flock in a concurrently inherited
        // descriptor. Explicitly release our acquired lock on every transaction exit.
        let _ = self.0.unlock();
    }
}
fn lock_error(error: TryLockError) -> String {
    match error {
        TryLockError::WouldBlock => "其他窗口正在更新最近工程，请稍后重试".into(),
        TryLockError::Error(error) => format!("无法取得最近工程写锁：{error}"),
    }
}

#[cfg(test)]
#[path = "write_lock_tests.rs"]
mod tests;
