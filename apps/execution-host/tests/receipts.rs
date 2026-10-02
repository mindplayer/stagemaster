mod support;
use serde_json::json;
use std::time::Duration;
use support::*;

#[test]
fn abandoned_http_reply_retains_outcome_and_lease_expiry_preserves_playback() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::start();
    let id = h.session().await;
    let acquired = h
        .command(
            &id,
            1,
            json!({"kind":"acquire","durationMs":1000,"takeover":false}),
        )
        .await;
    let step = ok(h.get("/source")).await["steps"][0]["id"].clone();
    let action = json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":{"kind":"start","step":step}});
    let response = h
        .post(&format!("/sessions/{id}/commands"))
        .json(&json!({"serial":"2","ttlMs":5000,"command":action}))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    drop(response); // Lose the accepted request's HTTP body; do not cancel/reissue the operation.
    let recovered = h.command(&id, 2, action).await;
    assert_eq!(recovered["kind"], "applied");
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let later = h.state().await;
    assert_eq!(later["status"], "Running");
    assert_eq!(later["instance"], recovered["state"]["instance"]);
    assert!(later["owner"].is_null());
    let refused = h
        .command(&id, 3, json!({"kind":"renew","durationMs":1000}))
        .await;
    assert_eq!(refused["code"], "Runtime(Lease)");
    let new_id = h.session().await;
    assert_eq!(h.acquire(&new_id, false).await["kind"], "acquired");
    h.close().await;
}
