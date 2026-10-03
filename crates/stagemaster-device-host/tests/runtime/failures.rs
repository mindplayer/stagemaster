use crate::{adapter::*, flows, runtime_support::rights};
use stagemaster_device_host::{Phase, ProblemCode, RuntimeIntent};
use stagemaster_runtime::{Action, Status};
use stagemaster_runtime_protocol::Operation;
use std::{task::Poll, time::Duration};

#[tokio::test(start_paused = true)]
async fn abandoned_started_call_preserves_uncertainty_and_does_not_stop_or_replay_the_program() {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    let step = flows::prepare(&host).await;
    let intent = RuntimeIntent {
        operation: Operation::Apply(Action::Start { step }),
        expected_revision: host
            .runtime_snapshot(epoch)
            .unwrap()
            .last_response
            .unwrap()
            .observed
            .revision,
    };
    data.lock().unwrap().hold_reply = true;
    let mut pending = Box::pin(host.exchange_runtime(epoch, intent));
    assert!(futures_util::poll!(&mut pending).is_pending());
    tokio::time::advance(Duration::from_millis(20)).await;
    settle().await;
    assert_eq!(
        data.lock().unwrap().observation.status,
        Some(Status::Running)
    );
    assert_eq!(
        host.exchange_runtime(epoch, intent).await.unwrap_err().code,
        ProblemCode::Busy
    );
    drop(pending);
    tokio::time::advance(Duration::from_millis(20)).await;
    wait_for_settled(&host).await;
    assert_eq!(status(&host).phase, Phase::Fault);
    let historical = host.runtime_snapshot(epoch).unwrap();
    assert!(historical.peer.is_none());
    assert_eq!(historical.pending, Some(intent));
    assert!(historical.last_response.is_some());
    let instance = data.lock().unwrap().observation.instance;
    assert_eq!(
        data.lock().unwrap().observation.status,
        Some(Status::Running)
    );
    data.lock().unwrap().hold_reply = false;
    let next = connect(&host, rights()).await;
    assert!(host.runtime_snapshot(next).unwrap().pending.is_none());
    command(&host, Operation::Status).await;
    {
        let s = data.lock().unwrap();
        assert_eq!(s.observation.instance, instance);
        assert_eq!(s.observation.owner, None);
        assert_eq!(
            s.commands
                .iter()
                .filter(|r| matches!(r.operation, Operation::Apply(Action::Start { .. })))
                .count(),
            1
        );
    }
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn keepalive_does_not_extend_the_services_total_reply_deadline() {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    data.lock().unwrap().hold_reply = true;
    let intent = RuntimeIntent {
        operation: Operation::Status,
        expected_revision: 0,
    };
    let mut pending = Box::pin(host.exchange_runtime(epoch, intent));
    assert!(futures_util::poll!(&mut pending).is_pending());
    settle().await;
    for _ in 0..14 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        settle().await;
        assert_eq!(status(&host).phase, Phase::Connected);
    }
    assert!(data.lock().unwrap().beats >= 14);
    tokio::time::sleep(Duration::from_secs(2)).await;
    settle().await;
    assert!(pending.await.is_err());
    assert!(host.runtime_snapshot(epoch).unwrap().peer.is_none());
    assert_eq!(host.runtime_snapshot(epoch).unwrap().pending, Some(intent));
    assert_eq!(data.lock().unwrap().commands.len(), 1);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cancelling_a_call_before_dispatch_does_not_send_it_or_break_the_connection() {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    let mut pending = Box::pin(host.exchange_runtime(
        epoch,
        RuntimeIntent {
            operation: Operation::Status,
            expected_revision: 0,
        },
    ));
    assert!(matches!(futures_util::poll!(&mut pending), Poll::Pending));
    drop(pending);
    settle().await;
    assert!(data.lock().unwrap().commands.is_empty());
    assert!(host.runtime_snapshot(epoch).unwrap().pending.is_none());
    assert_eq!(command(&host, Operation::Status).await.request.id, 1);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn mismatched_public_description_cannot_publish_an_admitted_runtime() {
    let (host, data) = setup();
    data.lock().unwrap().wrong_description = true;
    let epoch = connect(&host, rights()).await;
    assert_eq!(status(&host).phase, Phase::Fault);
    assert_eq!(status(&host).problem.unwrap().code, ProblemCode::Runtime);
    assert!(host.runtime_snapshot(epoch).unwrap().peer.is_none());
    assert!(data.lock().unwrap().commands.is_empty());
    assert_eq!(data.lock().unwrap().disconnects, 1);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cancelling_the_connection_during_send_keeps_the_registered_intent() {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    let fault = data.lock().unwrap().send_fault.clone();
    fault.store(1, std::sync::atomic::Ordering::Release);
    let intent = RuntimeIntent {
        operation: Operation::Status,
        expected_revision: 0,
    };
    let mut call = Box::pin(host.exchange_runtime(epoch, intent));
    assert!(futures_util::poll!(&mut call).is_pending());
    settle().await;
    assert_eq!(host.runtime_snapshot(epoch).unwrap().pending, Some(intent));
    disconnect(&host).await;
    assert_eq!(call.await.unwrap_err().code, ProblemCode::Lost);
    let observed = host.runtime_snapshot(epoch).unwrap();
    assert_eq!(observed.pending, Some(intent));
    assert!(observed.peer.is_none());
    assert!(data.lock().unwrap().commands.is_empty());
    let scan = host
        .request(stagemaster_device_host::Request::Scan { epoch })
        .unwrap();
    let old = host.runtime_snapshot(scan.epoch).unwrap();
    assert_eq!(old.connection_epoch, Some(epoch));
    assert_eq!(old.pending, Some(intent));
    host.request(stagemaster_device_host::Request::Cancel { epoch: scan.epoch })
        .unwrap();
    wait_for_settled(&host).await;
    fault.store(0, std::sync::atomic::Ordering::Release);
    let next = connect(&host, rights()).await;
    assert!(host.runtime_snapshot(next).unwrap().pending.is_none());
    assert_eq!(command(&host, Operation::Status).await.request.id, 1);
    host.shutdown().await.unwrap();
}
