use crate::{adapter::*, runtime_support::rights};
use serde_json::{Value, json};
use stagemaster_device_host::runtime_ui::{ExpectedAccess, Request};

async fn invoke(host: &Host, value: Value) -> Value {
    let request: Request = serde_json::from_value(value).unwrap();
    serde_json::to_value(
        host.runtime_request(request, &ExpectedAccess::default())
            .await
            .unwrap(),
    )
    .unwrap()
}
async fn apply(host: &Host, action: Value) -> Value {
    let epoch = status(host).epoch;
    let revision = host
        .runtime_snapshot(epoch)
        .unwrap()
        .last_response
        .unwrap()
        .observed
        .revision;
    invoke(
        host,
        json!({"kind":"apply", "epoch":epoch, "revision":revision.to_string(), "action":action}),
    )
    .await
}
fn state(view: &Value) -> &Value {
    assert!(view["reply"]["body"]["error"].is_null(), "{view}");
    &view["reply"]["body"]["state"]
}
#[tokio::test(start_paused = true)]
async fn json_application_controls_the_original_device_and_preserves_distinct_states() {
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    let initial = invoke(&host, json!({"kind":"refresh","epoch":epoch})).await;
    assert!(state(&initial)["owner"].is_null());
    assert!(state(&initial)["loaded"].is_null());
    let page = invoke(
        &host,
        json!({"kind":"catalog","epoch":epoch,"revision":initial["reply"]["revision"],"index":0}),
    )
    .await;
    let key = page["reply"]["body"]["program"]["key"].clone();
    let acquired = apply(&host, json!({"kind":"acquire","takeover":false})).await;
    assert!(state(&acquired)["owner"]["lease"].is_string());
    let selected = apply(&host, json!({"kind":"select","program":key})).await;
    assert_eq!(state(&selected)["selected"], key);
    assert!(state(&selected)["loaded"].is_null());
    let loaded = apply(&host, json!({"kind":"load"})).await;
    assert_eq!(state(&loaded)["loaded"], key);
    let step = invoke(
        &host,
        json!({"kind":"step","epoch":epoch,"revision":loaded["reply"]["revision"],"index":0}),
    )
    .await;
    let running = apply(
        &host,
        json!({"kind":"start","step":step["reply"]["body"]["step"]["id"]}),
    )
    .await;
    assert_eq!(state(&running)["status"], "running");
    assert!(state(&running)["instance"].is_string());
    let paused = apply(&host, json!({"kind":"pause"})).await;
    assert_eq!(state(&paused)["status"], "paused");
    let stale = invoke(&host,json!({"kind":"apply","epoch":epoch,"revision":initial["reply"]["revision"],"action":{"kind":"resume"}})).await;
    assert!(
        stale["reply"]["body"]["error"]
            .as_str()
            .unwrap()
            .contains("上下文")
    );
    assert!(stale["peer"].is_object());
    apply(&host, json!({"kind":"resume"})).await;
    disconnect(&host).await;
    let historical = invoke(&host, json!({"kind":"snapshot","epoch":epoch})).await;
    assert!(historical["peer"].is_null());
    assert!(!historical["lastResponse"].is_null());
    assert_eq!(
        data.lock().unwrap().observation.status,
        Some(stagemaster_runtime::Status::Running)
    );
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn malformed_json_or_missing_native_access_cannot_dispatch_or_escalate() {
    for bad in [
        json!({"kind":"apply","epoch":1,"revision":9_007_199_254_740_993_u64,"action":{"kind":"stop"}}),
        json!({"kind":"connect","epoch":1,"id":"x","control":true}),
        json!({"kind":"apply","epoch":1,"revision":"1","action":{"kind":"renew","duration":999_999}}),
    ] {
        assert!(
            serde_json::from_value::<Request>(bad.clone()).is_err(),
            "accepted {bad}"
        );
    }
    let (host, data) = setup();
    let epoch = connect(&host, rights()).await;
    for revision in ["", "01", "+1", " 1", "18446744073709551616"] {
        let request = serde_json::from_value(
            json!({"kind":"apply","epoch":epoch,"revision":revision,"action":{"kind":"stop"}}),
        )
        .unwrap();
        assert!(
            host.runtime_request(request, &ExpectedAccess::default())
                .await
                .is_err()
        );
    }
    for id in ["00".repeat(16), "é".repeat(16), "x".repeat(32)] {
        let request = serde_json::from_value(json!({"kind":"apply","epoch":epoch,"revision":"0","action":{"kind":"start","step":id}})).unwrap();
        assert!(
            host.runtime_request(request, &ExpectedAccess::default())
                .await
                .is_err()
        );
    }
    assert!(data.lock().unwrap().commands.is_empty());
    disconnect(&host).await;
    let request = Request::Connect {
        epoch,
        id: "runtime".into(),
    };
    assert!(
        host.runtime_request(request, &ExpectedAccess::default())
            .await
            .is_err()
    );
    assert_eq!(data.lock().unwrap().connects, 1);
    host.shutdown().await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn maintenance_is_explicit_and_renewal_stays_inside_fixed_admission() {
    let (host, _) = setup();
    let epoch = connect(
        &host,
        stagemaster_runtime_protocol::Access {
            installation: true,
            ..rights()
        },
    )
    .await;
    invoke(&host, json!({"kind":"refresh","epoch":epoch})).await;
    let acquired = apply(&host, json!({"kind":"acquire","takeover":false})).await;
    let lease = state(&acquired)["owner"]["lease"].clone();
    let renewed = apply(&host, json!({"kind":"renew"})).await;
    assert_eq!(state(&renewed)["owner"]["lease"], lease);
    let expires = state(&renewed)["owner"]["expiresMs"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let first_observed = acquired["reply"]["observedMs"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    // Device time already advanced during discovery. Compare in the same clock domain.
    assert!(expires < first_observed + 60_000);
    let maintenance = apply(&host, json!({"kind":"beginMaintenance"})).await;
    assert_eq!(state(&maintenance)["mode"], "quiescing");
    let cancelled = apply(&host, json!({"kind":"cancelMaintenance"})).await;
    assert_eq!(state(&cancelled)["mode"], "operation");
    let released = apply(&host, json!({"kind":"release"})).await;
    assert!(state(&released)["owner"].is_null());
    host.shutdown().await.unwrap();
}
