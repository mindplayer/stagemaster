use super::Bridge;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

#[tokio::test]
async fn bridge_close_is_idempotent_and_refuses_future_requests() {
    let bridge = Bridge::default();
    assert!(bridge.access().await.is_ok());
    bridge.close().await;
    bridge.close().await;
    assert!(bridge.access().await.is_err());
}

#[tokio::test]
async fn queued_request_cannot_reopen_bridge_after_shutdown_begins() {
    let bridge = Bridge::default();
    let held = bridge.runtime.lock().await;
    let mut queued = pin!(bridge.access());
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(queued.as_mut().poll(&mut context), Poll::Pending));
    bridge.begin_shutdown();
    // New work is rejected even while the runtime lock is still held.
    assert!(bridge.access().await.is_err());
    drop(held);
    assert!(queued.await.is_err());
    bridge.close().await;
}
