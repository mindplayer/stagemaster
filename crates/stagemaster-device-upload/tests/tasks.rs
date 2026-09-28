mod support;
use stagemaster_device_upload::{Phase, Prepared};
use stagemaster_transfer::Command;
use std::time::Duration;
use support::*;

#[tokio::test(start_paused = true)]
async fn actual_package_installs_with_authoritative_receipt_and_immutable_source() {
    let (host, state) = setup();
    let bytes = package();
    let original = bytes.clone();
    let initial = host
        .start(Prepared::new(bytes).unwrap(), 1, DEVICE)
        .unwrap();
    let task = initial.task.unwrap();
    assert!(task.running);
    assert_eq!(task.confirmed_bytes, 0);
    let final_view = finished(&host).await;
    assert!(final_view.revision > initial.revision);
    let final_task = final_view.task.unwrap();
    assert_eq!(final_task.phase, Phase::Installed);
    let receipt = final_task.receipt.unwrap();
    assert_eq!(receipt.digest, task.package.digest);
    assert_eq!(receipt.bytes, task.package.bytes);
    assert_eq!(receipt.generation, "1");
    {
        let s = state.lock().unwrap();
        let installed = s.server.snapshot().unwrap();
        let saved = std::fs::read(
            s.directory
                .path()
                .join(format!("slot-{}.smpkg", installed.commit().slot.index())),
        )
        .unwrap();
        assert_eq!(saved, original.as_ref());
    }
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn no_auth_wrong_device_and_capacity_never_start_or_send() {
    for failure in 0..5 {
        let (host, state) = setup();
        {
            let mut s = state.lock().unwrap();
            match failure {
                0 => s.allowed = false,
                1 => s.target.peer.device = [8; 16],
                2 => s.target.limits.package_bytes = 100,
                3 => s.target.limits.programs = 1,
                _ => s.target.limits.loader_bytes = 1,
            }
        }
        assert!(host.start(prepared(), 1, DEVICE).is_err());
        assert!(host.snapshot().unwrap().task.is_none());
        assert!(state.lock().unwrap().commands.is_empty());
        host.shutdown().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn one_unresolved_task_and_stale_actions_cannot_mutate_new_task() {
    let (host, _) = setup();
    let first = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    assert!(host.start(prepared(), 1, DEVICE).is_err());
    assert!(host.forget(&first.id).is_err());
    assert!(host.cancel("old-task").is_err());
    finished(&host).await;
    let second = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    assert_ne!(second.id, first.id);
    assert!(host.cancel(&first.id).is_err());
    assert!(host.resume(&first.id, 1).is_err());
    assert!(host.forget(&first.id).is_err());
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Installed);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn cancel_intent_waits_for_current_message_and_confirms_device_cancellation() {
    let (host, state) = setup();
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    loop {
        settle().await;
        if state.lock().unwrap().commands.contains(&Command::Write) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    let view = host.cancel(&task.id).unwrap().task.unwrap();
    assert!(view.running && view.cancel_requested);
    assert_eq!(view.phase, Phase::Cancelling);
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Cancelled);
    assert!(state.lock().unwrap().commands.contains(&Command::Cancel));
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn lost_commit_cancel_and_wrong_device_recovery_preserve_original_intent() {
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::LostCommit;
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    let lost = finished(&host).await.task.unwrap();
    assert_eq!(lost.phase, Phase::Reconnect);
    assert!(lost.receipt.is_none());
    assert!(host.start(prepared(), 1, DEVICE).is_err());
    assert!(
        host.cancel(&task.id)
            .unwrap()
            .task
            .unwrap()
            .cancel_requested
    );
    assert!(
        host.resume(&task.id, 1).is_err(),
        "uncertain session cannot be reused"
    );
    reconnect(&state);
    state.lock().unwrap().target.peer.device = [8; 16];
    let count = state.lock().unwrap().commands.len();
    assert!(host.resume(&task.id, 2).is_err());
    assert_eq!(state.lock().unwrap().commands.len(), count);
    state.lock().unwrap().target.peer.device = [1; 16];
    host.resume(&task.id, 2).unwrap();
    let done = finished(&host).await.task.unwrap();
    assert_eq!(done.phase, Phase::Installed);
    assert!(done.cancel_requested);
    assert_eq!(done.package.digest, task.package.digest);
    assert_eq!(done.receipt.unwrap().generation, "1");
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn known_remote_failure_can_retry_same_connection_without_duplicate_commit() {
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::RejectBegin;
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Failed);
    host.resume(&task.id, 1).unwrap();
    let done = finished(&host).await.task.unwrap();
    assert_eq!(done.phase, Phase::Installed);
    assert_eq!(done.receipt.unwrap().generation, "1");
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn waiting_cancel_survives_disconnect_and_never_claims_early_success() {
    let (host, state) = setup();
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    state.lock().unwrap().allowed = false;
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Reconnect);
    let pending = host.cancel(&task.id).unwrap().task.unwrap();
    assert!(pending.cancel_requested);
    assert_eq!(pending.phase, Phase::Reconnect);
    reconnect(&state);
    host.resume(&task.id, 2).unwrap();
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::NotStarted);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn shutdown_and_drop_abort_local_wait_without_claiming_remote_cancel() {
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::Hang;
    host.start(prepared(), 1, DEVICE).unwrap();
    settle().await;
    assert_eq!(state.lock().unwrap().inflight, 1);
    host.shutdown().await.unwrap();
    assert_eq!(state.lock().unwrap().inflight, 0);
    assert_eq!(
        host.snapshot().unwrap().task.unwrap().phase,
        Phase::Reconnect
    );
    assert!(!state.lock().unwrap().commands.contains(&Command::Cancel));
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::Hang;
    host.start(prepared(), 1, DEVICE).unwrap();
    settle().await;
    drop(host);
    settle().await;
    assert_eq!(state.lock().unwrap().inflight, 0);
}

#[tokio::test(start_paused = true)]
async fn adapter_panic_preserves_recoverable_task_and_explicit_forget_is_local_only() {
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::Panic;
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Reconnect);
    let commands = state.lock().unwrap().commands.clone();
    host.forget(&task.id).unwrap();
    assert!(host.snapshot().unwrap().task.is_none());
    assert_eq!(state.lock().unwrap().commands, commands);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn new_packages_share_confirmed_session_counter_and_advance_generation() {
    let (host, _) = setup();
    for (level, generation) in [(1000, "1"), (2000, "2"), (3000, "3")] {
        let prepared = Prepared::new(package_at(level)).unwrap();
        let expected = prepared.info().digest.clone();
        host.start(prepared, 1, DEVICE).unwrap();
        let done = finished(&host).await.task.unwrap();
        assert_eq!(done.phase, Phase::Installed);
        let receipt = done.receipt.unwrap();
        assert_eq!(receipt.digest, expected);
        assert_eq!(receipt.generation, generation);
    }
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn forgetting_uncertain_work_does_not_reset_live_session_counter() {
    let (host, state) = setup();
    state.lock().unwrap().fault = Fault::LostCommit;
    let task = host.start(prepared(), 1, DEVICE).unwrap().task.unwrap();
    finished(&host).await;
    host.forget(&task.id).unwrap();
    assert!(host.start(prepared(), 1, DEVICE).is_err());
    reconnect(&state);
    host.start(prepared(), 2, DEVICE).unwrap();
    assert_eq!(finished(&host).await.task.unwrap().phase, Phase::Installed);
    host.shutdown().await.unwrap();
}

#[test]
fn transfer_cursor_only_exists_between_confirmed_messages() {
    let source = package();
    let mut upload = stagemaster_transfer::Upload::new(source.as_ref()).unwrap();
    assert_eq!(upload.next_request_id(), None);
    assert!(upload.connect_at([3; 16], 0).is_err());
    upload.connect_at([3; 16], 9).unwrap();
    assert_eq!(upload.next_request_id(), Some(9));
    let frame = upload.outbound().unwrap().unwrap().clone();
    assert_eq!(upload.next_request_id(), None);
    assert_eq!(
        stagemaster_transfer::Request::decode(frame.bytes())
            .unwrap()
            .id,
        9
    );
    upload.disconnect();
    assert_eq!(upload.next_request_id(), None);
}
