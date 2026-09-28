use super::support::*;
use stagemaster_device_host::{Phase, ProblemCode as C};
use stagemaster_transfer::{Action, Request as WireRequest, Response};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn public_declaration_cannot_grant_permission() {
    let (host, state) = setup(20);
    state.lock().unwrap().grant = None;
    let connected = connect(&host).await;
    assert_eq!(connected.phase, Phase::Connected);
    assert!(host.installation_peer(connected.epoch).unwrap().is_none());
    let frame = WireRequest {
        link: [1; 16],
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap();
    assert_eq!(
        host.exchange_installation(connected.epoch, frame)
            .await
            .unwrap_err()
            .code,
        C::Installation
    );
    assert_eq!(state.lock().unwrap().fragments, 0);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn incompatible_authenticated_facts_fail_closed() {
    for field in 0..9 {
        let (host, state) = setup(20);
        {
            let mut s = state.lock().unwrap();
            match field {
                0 => s.description = None,
                1 => s.grant.as_mut().unwrap().device = [5; 16],
                2 => s.grant.as_mut().unwrap().boot = [5; 16],
                3 => s.grant.as_mut().unwrap().authentication = 2,
                4 => s.grant.as_mut().unwrap().fragment_bytes = 0,
                5 => s.grant.as_mut().unwrap().message_bytes = 1024,
                6 => s.description.as_mut().unwrap().limits.transfer_version = 2,
                7 => s.grant.as_mut().unwrap().fragment_bytes = 1281,
                _ => {
                    s.grant.as_mut().unwrap().message_bytes = 1281;
                    s.description.as_mut().unwrap().limits.message_bytes = 1281;
                }
            }
        }
        let connected = connect(&host).await;
        assert_eq!(connected.phase, Phase::Fault, "field {field}");
        assert_eq!(connected.problem.unwrap().code, C::Installation);
        assert!(connected.description.is_none());
        assert_eq!(state.lock().unwrap().fragments, 0);
        host.shutdown().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn busy_rejected_and_abandoning_active_request_disconnects() {
    let (host, state) = setup(20);
    let host = Arc::new(host);
    connect(&host).await;
    state.lock().unwrap().fault = Fault::NoReply;
    let frame = query(&host);
    let peer = host.clone();
    let epoch = status(&host).epoch;
    let task = tokio::spawn(async move { peer.exchange_installation(epoch, frame).await });
    settle().await;
    assert!(state.lock().unwrap().fragments > 0);
    assert_eq!(
        host.exchange_installation(epoch, query(&host))
            .await
            .unwrap_err()
            .code,
        C::Busy
    );
    task.abort();
    let _ = task.await;
    tokio::time::advance(Duration::from_millis(20)).await;
    settle().await;
    assert_eq!(status(&host).phase, Phase::Fault);
    assert_eq!(state.lock().unwrap().disconnects, 1);
    state.lock().unwrap().fault = Fault::None;
    connect(&host).await;
    host.exchange_installation(status(&host).epoch, query(&host))
        .await
        .unwrap();
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn abandoned_queued_request_never_starts_or_blocks_next_call() {
    let (host, state) = setup(20);
    connect(&host).await;
    let frame = query(&host);
    let epoch = status(&host).epoch;
    let mut future = Box::pin(host.exchange_installation(epoch, frame));
    assert!(futures_util::poll!(&mut future).is_pending());
    drop(future);
    settle().await;
    assert_eq!(state.lock().unwrap().fragments, 0);
    host.exchange_installation(epoch, query(&host))
        .await
        .unwrap();
    assert_eq!(status(&host).phase, Phase::Connected);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn corrupt_incomplete_and_lost_responses_close_connection() {
    for (fault, expected) in [
        (Fault::WrongBoot, C::Protocol),
        (Fault::WrongSession, C::Protocol),
        (Fault::WrongId, C::Protocol),
        (Fault::WrongCommand, C::Protocol),
        (Fault::Extra, C::Protocol),
        (Fault::EarlyReply, C::Protocol),
        (Fault::Empty, C::Protocol),
        (Fault::FailNotify, C::Lost),
        (Fault::FailWrite, C::Lost),
        (Fault::Partial, C::Timeout),
        (Fault::NoReply, C::Timeout),
    ] {
        let (host, state) = setup(20);
        connect(&host).await;
        state.lock().unwrap().fault = fault;
        let began = Instant::now();
        assert_eq!(
            host.exchange_installation(status(&host).epoch, query(&host))
                .await
                .unwrap_err()
                .code,
            expected
        );
        settle().await;
        assert_eq!(status(&host).phase, Phase::Fault);
        assert_eq!(state.lock().unwrap().disconnects, 1);
        if matches!(fault, Fault::Partial) {
            assert!(began.elapsed() <= Duration::from_millis(5020));
        }
        if matches!(fault, Fault::NoReply) {
            assert!(began.elapsed() <= Duration::from_millis(30020));
        }
        host.shutdown().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn stalled_fragment_and_slow_whole_frame_have_fixed_deadlines() {
    for (payload, delay, maximum) in [(20, 501, 501), (1, 300, 5020)] {
        let (host, state) = setup(payload);
        connect(&host).await;
        state.lock().unwrap().fragment_delay = Duration::from_millis(delay);
        let began = Instant::now();
        assert_eq!(
            host.exchange_installation(status(&host).epoch, query(&host))
                .await
                .unwrap_err()
                .code,
            C::Timeout
        );
        assert!(began.elapsed() <= Duration::from_millis(maximum));
        settle().await;
        assert_eq!(status(&host).phase, Phase::Fault);
        host.shutdown().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn application_error_is_a_correlated_response_not_transport_failure() {
    let (host, _state) = setup(20);
    let connected = connect(&host).await;
    let peer = host.installation_peer(connected.epoch).unwrap().unwrap();
    let frame = WireRequest {
        link: peer.session,
        id: 1,
        action: Action::Cancel(stagemaster_install::Transaction {
            boot: peer.boot,
            counter: 1,
        }),
    }
    .encode()
    .unwrap();
    let response = host
        .exchange_installation(connected.epoch, frame)
        .await
        .unwrap();
    assert!(Response::decode(response.bytes()).unwrap().result.is_err());
    assert_eq!(status(&host).phase, Phase::Connected);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn revoked_grant_and_disconnect_clear_queued_io() {
    let (host, state) = setup(20);
    let host = Arc::new(host);
    connect(&host).await;
    state.lock().unwrap().fault = Fault::NoReply;
    let owner = host.clone();
    let frame = query(&host);
    let epoch = status(&host).epoch;
    let task = tokio::spawn(async move { owner.exchange_installation(epoch, frame).await });
    settle().await;
    state.lock().unwrap().grant = None;
    tokio::time::advance(Duration::from_millis(20)).await;
    settle().await;
    assert_eq!(task.await.unwrap().unwrap_err().code, C::Installation);
    assert_eq!(status(&host).phase, Phase::Fault);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn stale_request_session_does_not_send_any_bytes() {
    let (host, state) = setup(20);
    let connected = connect(&host).await;
    let frame = WireRequest {
        link: [8; 16],
        id: 1,
        action: Action::Status,
    }
    .encode()
    .unwrap();
    assert_eq!(
        host.exchange_installation(connected.epoch, frame)
            .await
            .unwrap_err()
            .code,
        C::Protocol
    );
    assert_eq!(state.lock().unwrap().fragments, 0);
    assert_eq!(status(&host).phase, Phase::Connected);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn response_wait_keeps_heartbeats_running() {
    let (host, state) = setup(20);
    connect(&host).await;
    state.lock().unwrap().notification_delay = Duration::from_secs(10);
    host.exchange_installation(status(&host).epoch, query(&host))
        .await
        .unwrap();
    assert!(state.lock().unwrap().beats.len() >= 7);
    host.shutdown().await.unwrap();
}
