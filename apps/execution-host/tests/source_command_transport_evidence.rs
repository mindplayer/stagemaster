mod support;
use serde_json::Value;
use stagemaster_execution_client::{Action, BatchAction, Client, OutputAction};
use std::sync::atomic::Ordering;
use support::{client_observation::*, *};
fn evidence(client: &Client) -> Value {
    serde_json::to_value(client.view()).unwrap()["sourceOperation"].clone()
}
#[test]
fn no_control_invalid_host_and_capacity_are_not_submissions_or_old_success() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let view = client.view();
        let source = &view.catalog.sources[0];
        assert!(
            client
                .apply(&view.host_id, "0", &source.id, Action::Pause {})
                .await
                .is_err()
        );
        let no_control = evidence(&client);
        assert_eq!(no_control["attempted"], false);
        assert_eq!(no_control["notSubmittedReason"], "sendPreflight");
        assert!(no_control["serial"].is_null());
        let acquired = client.acquire(false).await;
        confirmed_control(&mut client, acquired, "acquired").await;
        assert!(
            client
                .apply(
                    &"private-host".repeat(10_000),
                    "1",
                    &source.id,
                    Action::Pause {}
                )
                .await
                .is_err()
        );
        let bad = evidence(&client);
        assert!(bad["target"].is_null());
        assert_eq!(bad["notSubmittedReason"], "invalidTarget");
        assert_eq!(bad["attempted"], false);
        assert!(
            client
                .apply(
                    &view.host_id,
                    "1",
                    &source.id,
                    Action::Start {
                        step: "private-step".repeat(10_000)
                    }
                )
                .await
                .is_err()
        );
        let large = evidence(&client);
        assert!(large["target"].is_null());
        assert_eq!(large["attempted"], false);
        assert_eq!(large["notSubmittedReason"], "sendPreflight");
        assert!(serde_json::to_vec(&large).unwrap().len() < 4096);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 1);
        task.abort();
        h.close().await;
    });
}
#[test]
fn http_refusal_and_not_retained_original_query_stay_separate_without_replay() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let acquired = client.acquire(false).await;
        confirmed_control(&mut client, acquired, "acquired").await;
        let view = client.view();
        proxy.reject_commands.store(true, Ordering::SeqCst);
        assert_eq!(
            client
                .apply(
                    &view.host_id,
                    &view.observation.snapshot.as_ref().unwrap().state.revision,
                    &view.catalog.sources[0].id,
                    Action::Pause {}
                )
                .await
                .unwrap_err(),
            BUSY
        );
        let submitted = evidence(&client);
        assert_eq!(submitted["submission"]["status"], 503);
        assert_eq!(submitted["submission"]["problem"], "httpRefused");
        assert!(submitted["receipt"].is_null());
        proxy.reject_commands.store(false, Ordering::SeqCst);
        assert!(client.refresh().await.is_err());
        let pending = evidence(&client);
        assert_eq!(pending["serial"], "2");
        assert_eq!(pending["submission"], submitted["submission"]);
        assert_eq!(pending["receiptRead"]["status"], 409);
        assert_eq!(pending["receiptRead"]["code"], "notRetained");
        assert!(
            client
                .apply(
                    &view.host_id,
                    "1",
                    &view.catalog.sources[0].id,
                    Action::Stop {}
                )
                .await
                .is_err()
        );
        assert_eq!(evidence(&client), pending);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert_eq!(proxy.forwarded_commands.load(Ordering::SeqCst), 1);
        task.abort();
        h.close().await;
    });
}

#[test]
fn explicit_resume_stop_replace_evidence_but_level_manual_output_batch_do_not() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        let result = client.acquire(false).await;
        let view = confirmed_control(&mut client, result, "acquired").await;
        let source = view.catalog.sources[0].clone();
        let result = client
            .apply(
                &view.host_id,
                &view.observation.snapshot.as_ref().unwrap().state.revision,
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await;
        confirmed_control(&mut client, result, "applied").await;
        for (action, kind, status) in [
            (Action::Pause {}, "pause", "Paused"),
            (Action::Resume {}, "resume", "Running"),
            (Action::Stop {}, "stop", "Idle"),
        ] {
            let before = client.view();
            let result = client
                .apply(
                    &before.host_id,
                    &before.observation.snapshot.as_ref().unwrap().state.revision,
                    &source.id,
                    action,
                )
                .await;
            let done = confirmed_control(&mut client, result, "applied").await;
            let value = evidence(&client);
            assert_eq!(value["target"]["action"]["kind"], kind);
            assert_eq!(value["receipt"]["sourceState"]["status"], status);
            assert_eq!(value["serial"], done.record.unwrap().serial);
        }
        let original = evidence(&client);
        let view = client.view();
        let result = client
            .apply(
                &view.host_id,
                &view.observation.snapshot.as_ref().unwrap().state.revision,
                &source.id,
                Action::Level { value: 32767 },
            )
            .await;
        confirmed_control(&mut client, result, "applied").await;
        assert_eq!(evidence(&client), original);
        let view = client.view();
        let result = client
            .apply(
                &view.host_id,
                &view.observation.snapshot.as_ref().unwrap().state.revision,
                &view.catalog.sources[2].id,
                Action::Stop {},
            )
            .await;
        confirmed_control(&mut client, result, "applied").await;
        assert_eq!(evidence(&client), original);
        let view = client.view();
        let result = client
            .output(
                &view.host_id,
                &view.observation.snapshot.as_ref().unwrap().state.revision,
                OutputAction::Level { percent: 50 },
            )
            .await;
        confirmed_control(&mut client, result, "applied").await;
        assert_eq!(evidence(&client), original);
        let view = client.view();
        let result = client
            .batch(
                &view.host_id,
                &view.observation.snapshot.as_ref().unwrap().state.revision,
                &[source.id],
                BatchAction::Stop {},
            )
            .await;
        confirmed_control(&mut client, result, "applied").await;
        assert_eq!(evidence(&client), original);
        h.close().await;
    });
}
