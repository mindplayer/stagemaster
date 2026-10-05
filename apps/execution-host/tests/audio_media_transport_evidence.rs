#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction, MediaCompletion};
use std::sync::atomic::Ordering;
use support::{client_audio::*, client_observation::confirmed_control, *};

#[test]
fn a_pending_non_media_request_cannot_relabel_the_previous_unsent_media_attempt() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let view = client.view();
        let state = &view.observation.snapshot.as_ref().unwrap().state;
        assert!(
            client
                .apply_media(
                    &view.host_id,
                    "01",
                    &state.media[0].id,
                    &state.media[0].generation,
                    MediaAction::Play {}
                )
                .await
                .is_err()
        );
        let original = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        proxy.corrupt.store(true, Ordering::SeqCst);
        assert_eq!(client.acquire(false).await.unwrap_err(), "后台响应格式无效");
        assert!(client.view().pending);
        assert!(
            client
                .apply_media(
                    &view.host_id,
                    &state.revision,
                    &state.media[0].id,
                    &state.media[0].generation,
                    MediaAction::Play {}
                )
                .await
                .is_err()
        );
        let retained = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 1);
        task.abort();
        h.close().await;
        assert_eq!(retained, original);
    });
}

#[test]
fn damaged_admitted_reply_retains_original_post_and_queries_only_the_same_serial() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let acquiring = client.acquire(false).await;
        confirmed_control(&mut client, acquiring, "acquired").await;
        let view = client.view();
        let state = &view.observation.snapshot.as_ref().unwrap().state;
        proxy.corrupt.store(true, Ordering::SeqCst);
        let error = client
            .apply_media(
                &view.host_id,
                &state.revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Seek {
                    position_ms: 2500,
                    playing: false,
                },
            )
            .await
            .unwrap_err();
        assert_eq!(error, "后台响应格式无效");
        let attempted = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert_eq!(attempted["serial"], "2");
        assert_eq!(attempted["submission"]["status"], 200);
        assert_eq!(attempted["submission"]["bodyComplete"], true);
        assert_eq!(attempted["submission"]["problem"], "invalidJson");
        assert!(attempted["receipt"].is_null());
        assert!(client.view().pending);
        // An extra explicit action is refused while this exact serial remains pending;
        // it must not replace the original request evidence or allocate another serial.
        assert!(
            client
                .apply_media(
                    &view.host_id,
                    &state.revision,
                    &state.media[0].id,
                    &state.media[0].generation,
                    MediaAction::Stop {}
                )
                .await
                .is_err()
        );
        assert_eq!(
            serde_json::to_value(client.view().media_operation.unwrap()).unwrap(),
            attempted
        );
        let completed = wait(&mut client, |v| {
            !v.pending
                && v.observation.snapshot.as_ref().unwrap().state.media[0]
                    .control
                    .as_ref()
                    .is_some_and(|c| c.status == MediaCompletion::Applied)
        })
        .await;
        let resolved = serde_json::to_value(completed.media_operation.unwrap()).unwrap();
        assert_eq!(resolved["target"], attempted["target"]);
        assert_eq!(resolved["serial"], "2");
        assert_eq!(resolved["submission"], attempted["submission"]);
        assert_eq!(resolved["receiptRead"]["status"], 200);
        assert_eq!(resolved["receipt"]["outcome"], "accepted");
        assert_eq!(resolved["receipt"]["mediaRequest"], "1");
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 2);
        assert!(proxy.receipt_reads.load(Ordering::SeqCst) > 0);
        task.abort();
        h.close().await;
    });
}

#[test]
fn admission_refusal_and_not_retained_query_stay_separate_without_replay() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let acquiring = client.acquire(false).await;
        confirmed_control(&mut client, acquiring, "acquired").await;
        let view = client.view();
        let state = &view.observation.snapshot.as_ref().unwrap().state;
        let before = h.state().await;
        proxy.reject_commands.store(true, Ordering::SeqCst);
        let error = client
            .apply_media(
                &view.host_id,
                &state.revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Play {},
            )
            .await
            .unwrap_err();
        assert_eq!(error, client_observation::BUSY);
        let submitted = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert_eq!(submitted["submission"]["status"], 503);
        assert_eq!(submitted["submission"]["problem"], "httpRefused");
        assert!(submitted["submission"]["code"].is_null()); // Non-JSON proxy error is not copied.
        proxy.reject_commands.store(false, Ordering::SeqCst);
        assert_eq!(
            client.refresh().await.unwrap_err(),
            "后台请求未成功（409），请核对连接与原回执"
        );
        let unresolved = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert_eq!(unresolved["submission"], submitted["submission"]);
        assert_eq!(unresolved["serial"], "2");
        assert_eq!(unresolved["receiptRead"]["status"], 409);
        assert_eq!(unresolved["receiptRead"]["code"], "notRetained");
        assert!(unresolved["receipt"].is_null());
        assert!(client.view().pending);
        assert!(client.release().await.is_err());
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 1);
        let after = h.state().await;
        assert_eq!(after["revision"], before["revision"]);
        assert_eq!(after["audio"], before["audio"]);
        assert_eq!(after["media"], before["media"]);
        task.abort();
        h.close().await;
    });
}
