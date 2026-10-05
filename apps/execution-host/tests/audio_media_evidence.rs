#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction};
use std::sync::atomic::Ordering;
use support::{client_audio::*, client_observation::confirmed_control, *};

#[test]
fn explicit_media_target_and_original_receipt_remain_identifiable() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let mut client = Client::open(&h.directory.path().join("run/discovery.json"))
            .await
            .unwrap();
        let acquiring = client.acquire(false).await;
        confirmed_control(&mut client, acquiring, "acquired").await;
        let paused = apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 2500,
                playing: false,
            },
        )
        .await;
        let state = &paused.observation.snapshot.as_ref().unwrap().state;
        let original = json!({"audio":state.audio,"media":state.media});
        let target = json!({
            "hostId":paused.host_id,"revision":state.revision,
            "group":state.media[0].id,"generation":state.media[0].generation,
            "action":{"kind":"exitLoop","instance":state.audio.as_ref().unwrap().instance,
                "region":0,"pass":"2","requested":true}
        });
        let submitting = client
            .apply_media(
                &paused.host_id,
                &state.revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::ExitLoop {
                    instance: state.audio.as_ref().unwrap().instance.clone().unwrap(),
                    region: 0,
                    pass: "2".into(),
                    requested: true,
                },
            )
            .await;
        let completed = confirmed_control(&mut client, submitting, "rejected").await;
        let current = &completed.observation.snapshot.as_ref().unwrap().state;
        assert_eq!(
            json!({"audio":current.audio,"media":current.media}),
            original
        );
        assert_eq!(
            completed
                .record
                .as_ref()
                .unwrap()
                .outcome
                .as_ref()
                .unwrap()
                .code
                .as_deref(),
            Some("loopTargetChanged")
        );
        let raw = serde_json::to_value(&completed).unwrap();
        let serial = completed.record.as_ref().unwrap().serial.clone();
        h.close().await;
        println!(
            "AUDIO023_ORIGINAL {}",
            json!({"target":target,"record":raw["record"],"evidence":raw["mediaOperation"]})
        );
        assert_eq!(raw["mediaOperation"]["target"], target);
        assert_eq!(raw["mediaOperation"]["serial"], serial);
        assert_eq!(raw["mediaOperation"]["attempted"], true);
        assert_eq!(raw["mediaOperation"]["submission"]["status"], 200);
        assert_eq!(raw["mediaOperation"]["receipt"]["outcome"], "rejected");
        assert_eq!(
            raw["mediaOperation"]["receipt"]["code"],
            "loopTargetChanged"
        );
    });
}

#[test]
fn maintenance_and_later_non_media_operations_do_not_erase_the_original_target() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let acquiring = client.acquire(false).await;
        confirmed_control(&mut client, acquiring, "acquired").await;
        apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 2500,
                playing: false,
            },
        )
        .await;
        let state = client.view().observation.snapshot.unwrap().state;
        let instance = state.audio.unwrap().instance.unwrap();
        let exited = apply(
            &mut client,
            MediaAction::ExitLoop {
                instance: instance.clone(),
                region: 0,
                pass: "1".into(),
                requested: true,
            },
        )
        .await;
        assert!(
            exited
                .observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .audio
                .as_ref()
                .unwrap()
                .loop_state
                .as_ref()
                .unwrap()
                .exit_requested
        );
        let original = serde_json::to_value(exited.media_operation.unwrap()).unwrap();
        renew_without_reposting_media(&h, &mut client, &proxy).await;
        assert_eq!(
            serde_json::to_value(client.view().media_operation.unwrap()).unwrap(),
            original
        );
        let serial = original["serial"].as_str().unwrap();
        let session = exited.session_id.as_ref().unwrap();
        let retired = h
            .get(&format!("/sessions/{session}/receipts/{serial}"))
            .send()
            .await
            .unwrap();
        assert_eq!(retired.status(), reqwest::StatusCode::CONFLICT);
        assert_eq!(
            retired.json::<serde_json::Value>().await.unwrap()["code"],
            "notRetained"
        );
        let cancelled = apply(
            &mut client,
            MediaAction::ExitLoop {
                instance,
                region: 0,
                pass: "1".into(),
                requested: false,
            },
        )
        .await;
        let evidence = serde_json::to_value(cancelled.media_operation.unwrap()).unwrap();
        assert_eq!(evidence["target"]["action"]["requested"], false);
        assert_ne!(evidence["serial"], original["serial"]);
        assert_eq!(evidence["receipt"]["outcome"], "accepted");
        assert!(
            !cancelled
                .observation
                .snapshot
                .unwrap()
                .state
                .audio
                .unwrap()
                .loop_state
                .unwrap()
                .exit_requested
        );
        let releasing = client.release().await;
        confirmed_control(&mut client, releasing, "released").await;
        assert_eq!(
            serde_json::to_value(client.view().media_operation.unwrap()).unwrap(),
            evidence
        );
        let reopened = Client::open(&path).await.unwrap();
        assert!(reopened.view().media_operation.is_none());
        task.abort();
        h.close().await;
    });
}

async fn renew_without_reposting_media(
    h: &Harness,
    client: &mut Client,
    proxy: &client_proxy::Proxy,
) {
    let requests = proxy.submissions.load(Ordering::SeqCst);
    let mut snapshot = ok(h.get("/state")).await;
    // Owned observation fault injection triggers the unchanged 30-second maintenance
    // threshold; the actual host lease and production deadlines are not shortened.
    let observed = snapshot["snapshot"]["state"]["observedMs"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    snapshot["snapshot"]["state"]["owner"]["expiresMs"] = (observed + 10000).to_string().into();
    *proxy.old_observation.lock().unwrap() = Some(snapshot);
    client.maintain().await.unwrap();
    settled(client).await;
    *proxy.old_observation.lock().unwrap() = None;
    assert_eq!(proxy.submissions.load(Ordering::SeqCst), requests + 1);
}

#[test]
fn invalid_input_and_send_preflight_do_not_allocate_a_serial_or_send_a_command() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let before = client.view();
        let state = before.observation.snapshot.as_ref().unwrap().state.clone();
        let error = client
            .apply_media(
                &before.host_id,
                &state.revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Play {},
            )
            .await
            .unwrap_err();
        assert_eq!(error, "请先取得运行控制权");
        let not_sent = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert_eq!(not_sent["notSubmittedReason"], "sendPreflight");
        assert_eq!(not_sent["attempted"], false);
        assert!(not_sent["serial"].is_null());
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 0);
        let acquiring = client.acquire(false).await;
        confirmed_control(&mut client, acquiring, "acquired").await;
        let paused = apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 2500,
                playing: false,
            },
        )
        .await;
        let state = &paused.observation.snapshot.as_ref().unwrap().state;
        let error = client
            .apply_media(
                &paused.host_id,
                "01",
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::ExitLoop {
                    instance: "Authorization: private-input".repeat(10000),
                    region: 0,
                    pass: "1".into(),
                    requested: true,
                },
            )
            .await
            .unwrap_err();
        assert!(!error.is_empty());
        let raw = serde_json::to_value(client.view().media_operation.unwrap()).unwrap();
        assert!(raw["target"].is_null());
        assert!(raw["serial"].is_null());
        assert_eq!(raw["notSubmittedReason"], "invalidTarget");
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert!(!raw.to_string().contains("Authorization"));
        let exit = apply(
            &mut client,
            MediaAction::ExitLoop {
                instance: state.audio.as_ref().unwrap().instance.clone().unwrap(),
                region: 0,
                pass: "1".into(),
                requested: true,
            },
        )
        .await;
        assert_eq!(exit.media_operation.unwrap().serial.as_deref(), Some("3"));
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 3);
        task.abort();
        h.close().await;
    });
}
