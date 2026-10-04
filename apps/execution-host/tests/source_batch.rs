mod support;
use serde_json::{Value, json};
use stagemaster_execution_client::{BatchAction, Client, View};
use support::{group::*, *};

fn command(revision: &Value, sources: Value, action: Value) -> Value {
    let batch = Value::Object(serde_json::Map::from_iter([
        ("kind".into(), "batch".into()),
        ("sources".into(), sources),
        ("action".into(), action),
    ]));
    json!({"kind":"submit","expectedRevision":revision,"action":batch})
}
#[test]
fn real_process_batch_pauses_resumes_and_stops_with_one_receipt_manual_retained() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let catalog = ok(h.get("/source")).await;
        assert!(catalog["capabilities"].as_array().unwrap().contains(&json!("sourceBatch")));
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let a = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
        let b = start(&h, &session, 3, &a["state"]["revision"], 1).await;
        let manual = apply(&h, &session, 4, &b["state"]["revision"], 3, json!({"kind":"patch","changes":[{"fixtureId":catalog["fixtures"][0]["id"],"attribute":"dimmer","value":{"kind":"normalized","value":0}}]})).await;
        let request = command(&manual["state"]["revision"], json!([id(1), id(2)]), json!({"kind":"pause"}));
        let paused = h.command(&session, 5, request.clone()).await;
        assert_eq!(paused["kind"], "applied");
        let revision = manual["state"]["revision"].as_str().unwrap().parse::<u64>().unwrap();
        assert_eq!(paused["state"]["revision"], (revision + 1).to_string());
        for index in [0, 1] { assert_eq!(paused["state"]["sources"][index]["status"], "Paused"); }
        let before = paused["state"]["sources"][0]["progress"].clone();
        let later = until(&h, |s|s["state"]["observedMs"] != paused["state"]["observedMs"]).await;
        assert_eq!(later["state"]["sources"][0]["progress"], before);
        assert_eq!(h.command(&session, 5, request).await, paused);
        let resumed = h.command(&session, 6, command(&paused["state"]["revision"], json!([id(1), id(2)]), json!({"kind":"resume"}))).await;
        for index in [0, 1] { assert_eq!(resumed["state"]["sources"][index]["status"], "Running"); }
        let stopped = h.command(&session, 7, command(&resumed["state"]["revision"], json!([id(1)]), json!({"kind":"stop"}))).await;
        assert_eq!(stopped["state"]["sources"][0]["status"], "Idle");
        assert_eq!(stopped["state"]["sources"][1]["status"], "Running");
        assert_eq!(stopped["state"]["sources"][2]["heldValues"], json!([0]));
        assert_eq!(stopped["state"]["output"], json!({"percent":100,"blackout":false}));
        h.close().await;
    });
}

#[test]
fn malformed_batches_reject_before_admission_and_unknown_or_manual_targets_never_partially_apply() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let running = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
        let too_large = h.post(&format!("/sessions/{session}/commands")).json(&json!({"serial":"3","ttlMs":5000,"command":command(&running["state"]["revision"], json!([id(1)]), json!({"kind":"stop"})), "padding":"x".repeat(8192)})).send().await.unwrap();
        assert_eq!(too_large.status(), 413);
        for (ids, action, semantic_rejection) in [
            (json!([]), json!({"kind":"stop"}), true), (json!([id(1),id(1)]), json!({"kind":"stop"}), true), (json!(vec![id(1);65]), json!({"kind":"stop"}), true),
            (json!(["bad"]), json!({"kind":"stop"}), true), (json!([id(1)]), json!({"kind":"start","step":id(1)}), false), (json!([id(1)]), json!({"kind":"stop","extra":true}), false),
        ] {
            let response = h.post(&format!("/sessions/{session}/commands")).json(&json!({"serial":"3","ttlMs":5000,"command":command(&running["state"]["revision"],ids,action)})).send().await.unwrap();
            assert_eq!(response.status(), 422);
            if semantic_rejection { assert_eq!(response.json::<Value>().await.unwrap()["code"], "invalid"); }
        }
        let mut serial = 3;
        for ids in [json!([id(1),id(3)]), json!([id(1),id(99)])] {
            let rejected = h.command(&session, serial, command(&running["state"]["revision"], ids, json!({"kind":"stop"}))).await;
            assert_eq!(rejected["kind"], "rejected");
            assert_eq!(h.state().await["sources"][0]["status"], "Running");
            assert_eq!(h.state().await["revision"], running["state"]["revision"]);
            serial += 1;
        }
        let valid = h.command(&session, serial, command(&running["state"]["revision"], json!([id(1),id(2)]), json!({"kind":"pause"}))).await;
        assert_eq!(valid["kind"], "applied");
        assert_eq!(valid["state"]["sources"][0]["status"], "Paused");
        assert_eq!(valid["state"]["sources"][1]["status"], "Idle");
        h.close().await;
    });
}

#[test]
fn single_program_v1_rejects_a_group_batch_without_changing_the_player() {
    runtime().block_on(async {
        let mut h = Harness::start();
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let before = h.state().await;
        let rejected = h
            .command(
                &session,
                2,
                command(
                    &acquired["state"]["revision"],
                    json!([id(1)]),
                    json!({"kind":"stop"}),
                ),
            )
            .await;
        assert_eq!(rejected["kind"], "rejected");
        assert_eq!(h.state().await["revision"], before["revision"]);
        assert_eq!(h.state().await["status"], before["status"]);
        h.close().await;
    });
}

async fn settled(client: &mut Client) -> View {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
    loop {
        let view = client.refresh().await.unwrap();
        if !view.pending {
            return view;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
#[test]
fn shared_client_preserves_single_serial_stale_revision_and_takeover_protection() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        client.acquire(false).await.unwrap();
        let initial = settled(&mut client).await;
        let revision = &initial
            .observation
            .snapshot
            .as_ref()
            .unwrap()
            .state
            .revision;
        for ids in [
            vec![],
            vec![id(1), id(1)],
            vec![id(1), id(3)],
            vec![id(1), id(99)],
            vec![id(1); 65],
        ] {
            assert!(
                client
                    .batch(&initial.host_id, revision, &ids, BatchAction::Stop {})
                    .await
                    .is_err()
            );
            assert!(!client.view().pending);
            assert_eq!(client.view().record.unwrap().serial, "1");
        }
        assert!(
            client
                .batch("old", revision, &[id(1)], BatchAction::Stop {})
                .await
                .is_err()
        );
        assert!(
            client
                .batch(&initial.host_id, "01", &[id(1)], BatchAction::Stop {})
                .await
                .is_err()
        );
        client
            .batch(
                &initial.host_id,
                revision,
                &[id(1), id(2)],
                BatchAction::Pause {},
            )
            .await
            .unwrap();
        let applied = settled(&mut client).await;
        assert_eq!(applied.record.as_ref().unwrap().serial, "2");
        assert_eq!(applied.record.unwrap().outcome.unwrap().kind, "applied");
        client
            .batch(
                &initial.host_id,
                revision,
                &[id(1), id(2)],
                BatchAction::Stop {},
            )
            .await
            .unwrap();
        assert_eq!(
            settled(&mut client)
                .await
                .record
                .unwrap()
                .outcome
                .unwrap()
                .kind,
            "rejected"
        );
        let mut other = Client::open(&path).await.unwrap();
        other.acquire(true).await.unwrap();
        settled(&mut other).await;
        let current = settled(&mut client).await;
        client
            .batch(
                &initial.host_id,
                &current.observation.snapshot.unwrap().state.revision,
                &[id(1)],
                BatchAction::Stop {},
            )
            .await
            .unwrap();
        assert_eq!(
            settled(&mut client)
                .await
                .record
                .unwrap()
                .outcome
                .unwrap()
                .kind,
            "rejected"
        );
        drop(client);
        drop(other);
        h.close().await;
    });
}
