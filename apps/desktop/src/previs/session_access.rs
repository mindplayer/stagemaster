//! Short, bounded acquisition for UI commands; renderer frame requests still fail fast.
use std::sync::{Mutex, TryLockError};
use std::time::{Duration, Instant};

pub(super) async fn access<T, R>(
    shared: &Mutex<T>,
    work: impl FnOnce(&mut T) -> Result<R, String>,
) -> Result<R, String> {
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        // No guard crosses an await, and the mutation is called at most once.
        match shared.try_lock() {
            Ok(mut value) => return work(&mut value),
            Err(TryLockError::Poisoned(_)) => {
                return Err("工程会话发生错误，请重启应用".into());
            }
            Err(TryLockError::WouldBlock) => {}
        }
        let now = Instant::now();
        if now >= deadline {
            return Err("工程仍在处理其他操作，请完成当前操作后重试".into());
        }
        tokio::time::sleep(Duration::from_millis(5).min(deadline - now)).await;
        if Instant::now() >= deadline {
            return Err("工程仍在处理其他操作，请完成当前操作后重试".into());
        }
    }
}

#[cfg(test)]
#[path = "session_access_tests.rs"]
mod tests;
