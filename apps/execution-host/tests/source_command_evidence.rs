mod support;
use serde_json::Value;
use stagemaster_execution_client::{Action, Client, View};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use support::*;

async fn settled(client: &mut Client, result: Result<View, String>) -> View {
    let mut view = result.unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    while view.pending {
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
        view = client.refresh().await.unwrap();
    }
    view
}
fn evidence(view: &View) -> Value {
    serde_json::to_value(view).unwrap()["sourceOperation"].clone()
}
async fn start(client: &mut Client) -> View {
    let view = client.view();
    let state = &view.observation.snapshot.as_ref().unwrap().state;
    let source = &view.catalog.sources[0];
    let result = client
        .apply(
            &view.host_id,
            &state.revision,
            &source.id,
            Action::Start {
                step: source.steps[0].id.clone(),
            },
        )
        .await;
    settled(client, result).await
}
#[test]
fn original_pause_target_and_receipt_survive_maintenance_and_other_actions() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        assert!(evidence(&client.view()).is_null());
        let result = client.acquire(false).await;
        settled(&mut client, result).await;
        let running = start(&mut client).await;
        let source = &running.catalog.sources[0];
        let revision = &running
            .observation
            .snapshot
            .as_ref()
            .unwrap()
            .state
            .revision;
        let result = client
            .apply(&running.host_id, revision, &source.id, Action::Pause {})
            .await;
        let paused = settled(&mut client, result).await;
        let original = evidence(&paused);
        assert_eq!(original["target"]["source"], source.id);
        assert_eq!(original["target"]["revision"], *revision);
        assert_eq!(original["target"]["action"]["kind"], "pause");
        assert_eq!(original["receipt"]["outcome"], "applied");
        assert_eq!(original["receipt"]["sourceState"]["status"], "Paused");
        // Only the read-side lease projection is shortened; the original host receives real renew.
        let mut observed = ok(h.get("/state")).await;
        let now = observed["snapshot"]["state"]["observedMs"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        observed["snapshot"]["state"]["owner"]["expiresMs"] = (now + 100).to_string().into();
        *proxy.old_observation.lock().unwrap() = Some(observed);
        let result = client.maintain().await;
        let maintained = settled(&mut client, result).await;
        assert_eq!(evidence(&maintained), original);
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 4);
        *proxy.old_observation.lock().unwrap() = None;
        let result = client.release().await;
        let released = settled(&mut client, result).await;
        assert_eq!(evidence(&released), original);
        assert!(evidence(&Client::open(&path).await.unwrap().view()).is_null());
        task.abort();
        h.close().await;
    });
}
#[test]
fn damaged_reply_only_queries_original_source_serial_without_replay() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let (proxy, task) = client_proxy::install(&h, &path).await;
        let mut client = Client::open(&path).await.unwrap();
        let result = client.acquire(false).await;
        settled(&mut client, result).await;
        let view = client.view();
        let source = &view.catalog.sources[0];
        proxy.corrupt.store(true, Ordering::SeqCst);
        assert_eq!(
            client
                .apply(
                    &view.host_id,
                    &view.observation.snapshot.as_ref().unwrap().state.revision,
                    &source.id,
                    Action::Start {
                        step: source.steps[0].id.clone()
                    }
                )
                .await
                .unwrap_err(),
            "后台响应格式无效"
        );
        let original = evidence(&client.view());
        assert_eq!(original["serial"], "2");
        assert_eq!(original["submission"]["problem"], "invalidJson");
        assert!(original["receipt"].is_null());
        assert!(
            client
                .apply(&view.host_id, "1", &source.id, Action::Pause {})
                .await
                .is_err()
        );
        assert_eq!(evidence(&client.view()), original);
        let view = client.refresh().await;
        let done = settled(&mut client, view).await;
        let resolved = evidence(&done);
        assert_eq!(resolved["serial"], "2");
        assert_eq!(resolved["target"], original["target"]);
        assert_eq!(resolved["submission"], original["submission"]);
        assert_eq!(resolved["receiptRead"]["status"], 200);
        assert_eq!(resolved["receipt"]["outcome"], "applied");
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
        assert!(proxy.receipt_reads.load(Ordering::SeqCst) > 0);
        task.abort();
        h.close().await;
    });
}
#[test]
fn stale_revision_cannot_borrow_source_state_from_rejected_receipt() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        let result = client.acquire(false).await;
        settled(&mut client, result).await;
        let running = start(&mut client).await;
        let source = &running.catalog.sources[0];
        let result = client
            .apply(&running.host_id, "0", &source.id, Action::Pause {})
            .await;
        let rejected = settled(&mut client, result).await;
        let evidence = evidence(&rejected);
        assert_eq!(evidence["receipt"]["outcome"], "rejected");
        assert!(evidence["receipt"]["sourceState"].is_null());
        assert_eq!(
            rejected
                .observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .sources[0]
                .status
                .as_deref(),
            Some("Running")
        );
        h.close().await;
    });
}
