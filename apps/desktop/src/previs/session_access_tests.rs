use super::access;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
}

#[test]
fn short_contention_yields_to_runtime_and_mutates_once() {
    let shared = Arc::new(Mutex::new(0));
    let held = shared.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let holder = std::thread::spawn(move || {
        let _guard = held.lock().unwrap();
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    ready_rx.recv().unwrap();
    runtime().block_on(async {
        // This timer cannot progress if acquisition blocks the single runtime thread.
        let release = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(15)).await;
            release_tx.send(()).unwrap();
        });
        let result = access(&shared, |value| {
            *value += 1;
            Ok(*value)
        })
        .await;
        release.await.unwrap();
        assert_eq!(result.unwrap(), 1);
    });
    holder.join().unwrap();
    assert_eq!(*shared.lock().unwrap(), 1);
}

#[test]
fn timeout_does_not_leave_a_queued_mutation() {
    let shared = Mutex::new(0);
    let guard = shared.lock().unwrap();
    let result = runtime().block_on(access(&shared, |value| {
        *value = 1;
        Ok(())
    }));
    assert!(result.unwrap_err().contains("请完成当前操作后重试"));
    drop(guard);
    assert_eq!(*shared.lock().unwrap(), 0);
}

#[test]
fn domain_errors_are_preserved_and_never_retried() {
    let shared = Mutex::new(0);
    let result = runtime().block_on(access(&shared, |value| {
        *value += 1;
        Err::<(), _>("工程已切换".into())
    }));
    assert_eq!(result.unwrap_err(), "工程已切换");
    assert_eq!(*shared.lock().unwrap(), 1);
}

#[test]
fn poisoned_session_is_distinct_from_temporary_contention() {
    let shared = Mutex::new(0);
    let _ = std::panic::catch_unwind(|| {
        let _guard = shared.lock().unwrap();
        panic!("故障注入");
    });
    let result = runtime().block_on(access(&shared, |_| Ok(())));
    assert!(result.unwrap_err().contains("工程会话发生错误"));
}
