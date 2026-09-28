use super::support::*;
use stagemaster_device_host::{Phase, ProblemCode as C};
use std::{sync::Arc, time::Duration};

#[tokio::test(start_paused = true)]
async fn heartbeat_cannot_publish_freshness_after_previous_deadline() {
    let (host, state) = setup(20);
    connect(&host).await;
    // Resume late but before expiration; the diagnostic read crosses the remaining budget.
    state.lock().unwrap().diagnostic_delay = Duration::from_millis(600);
    tokio::time::advance(Duration::from_secs(4)).await;
    settle().await;
    tokio::time::advance(Duration::from_millis(501)).await;
    settle().await;
    let snapshot = status(&host);
    assert_eq!(snapshot.phase, Phase::Fault);
    assert_eq!(snapshot.problem.unwrap().code, C::Timeout);
    assert!(snapshot.description.is_none());
    assert_eq!(snapshot.heartbeat_count, 0);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cancelled_connection_drops_active_request_and_clears_permission() {
    let (host, state) = setup(20);
    let host = Arc::new(host);
    let connected = connect(&host).await;
    state.lock().unwrap().fault = Fault::NoReply;
    let owner = host.clone();
    let frame = query(&host);
    let task =
        tokio::spawn(async move { owner.exchange_installation(connected.epoch, frame).await });
    settle().await;
    disconnect(&host).await;
    assert_eq!(task.await.unwrap().unwrap_err().code, C::Lost);
    assert!(host.installation_peer(connected.epoch).is_err());
    assert!(state.lock().unwrap().notifications.is_empty());
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn queued_request_counts_towards_single_inflight_budget() {
    let (host, state) = setup(20);
    connect(&host).await;
    let epoch = status(&host).epoch;
    let mut queued = Box::pin(host.exchange_installation(epoch, query(&host)));
    assert!(futures_util::poll!(&mut queued).is_pending());
    assert_eq!(
        host.exchange_installation(epoch, query(&host))
            .await
            .unwrap_err()
            .code,
        C::Busy
    );
    assert_eq!(state.lock().unwrap().fragments, 0);
    drop(queued);
    settle().await;
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn unsolicited_old_response_is_not_reused_for_a_new_request() {
    let (host, state) = setup(20);
    let connected = connect(&host).await;
    state.lock().unwrap().notifications.push_back(vec![1, 2, 3]);
    assert!(
        host.exchange_installation(connected.epoch, query(&host))
            .await
            .is_err()
    );
    settle().await;
    assert_eq!(status(&host).problem.unwrap().code, C::Protocol);
    assert_eq!(state.lock().unwrap().fragments, 0);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn drip_fed_response_does_not_extend_whole_response_deadline() {
    let (host, state) = setup(20);
    let host = Arc::new(host);
    connect(&host).await;
    state.lock().unwrap().fault = Fault::NoReply;
    let owner = host.clone();
    let frame = query(&host);
    let epoch = status(&host).epoch;
    let task = tokio::spawn(async move { owner.exchange_installation(epoch, frame).await });
    settle().await;
    // A valid response prefix; further bytes arrive every second, never renewing its deadline.
    for byte in [9, 0, 0, 55, 0x53, 0x54] {
        state.lock().unwrap().notifications.push_back(vec![byte]);
        tokio::time::advance(Duration::from_secs(1)).await;
        settle().await;
    }
    assert_eq!(task.await.unwrap().unwrap_err().code, C::Timeout);
    assert_eq!(status(&host).phase, Phase::Fault);
    host.shutdown().await.unwrap();
}
