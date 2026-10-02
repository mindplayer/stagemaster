mod support;
use serde_json::{Value, json};
use support::{group::*, *};

#[test]
fn source_control_manual_functions_and_atomic_refusals_share_one_process() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::start_group();
    assert_eq!(h.discovery["protocol"], 2);
    let directory = ok(h.get("/source")).await;
    assert_eq!(directory["sources"].as_array().unwrap().len(), 3);
    assert_eq!(directory["physicalOutput"], false);
    let session = h.session().await;
    let acquired = h.acquire(&session, false).await;
    let one = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
    let two = start(&h, &session, 3, &one["state"]["revision"], 1).await;
    assert_eq!(two["kind"], "applied");
    let baseline = until(&h, |s| s["frame"]["slots"][1] == 40).await;
    let fixture = directory["fixtures"][0]["id"].clone();
    let edit = |attribute: &str, value: Value| json!({"fixtureId":fixture,"attribute":attribute,"value":value});
    let red = edit(
        "color-wheel",
        json!({"kind":"function","functionKey":"red","position":0}),
    );
    let patch = json!({"kind":"patch","changes":[red.clone()]});
    let manual = apply(&h, &session, 4, &two["state"]["revision"], 3, patch.clone()).await;
    assert_eq!(manual["kind"], "applied");
    until(&h, |s| s["frame"]["slots"][1] == 20).await;
    assert_eq!(
        apply(&h, &session, 4, &two["state"]["revision"], 3, patch).await,
        manual
    );
    let mut serial = 5;
    for invalid in [
        json!({"kind":"patch","changes":[red.clone(),red]}),
        json!({"kind":"patch","changes":[edit("dimmer",json!({"kind":"normalized","value":65535})),edit("color-wheel",json!({"kind":"normalized","value":30000}))]}),
        json!({"kind":"patch","changes":[edit("color-wheel",json!({"kind":"function","functionKey":"missing","position":0}))]}),
        json!({"kind":"patch","changes":[edit("missing",json!({"kind":"release"}))]}),
        json!({"kind":"start","step":id(99)}),
    ] {
        let refused = apply(
            &h,
            &session,
            serial,
            &manual["state"]["revision"],
            3,
            invalid,
        )
        .await;
        assert_eq!(refused["kind"], "rejected");
        assert_eq!(h.state().await["revision"], manual["state"]["revision"]);
        serial += 1;
    }
    let absent = apply(
        &h,
        &session,
        serial,
        &manual["state"]["revision"],
        99,
        json!({"kind":"stop"}),
    )
    .await;
    assert_eq!(absent["kind"], "rejected");
    serial += 1;
    let held = until(&h, |s| {
        s["frame"]["slots"][0] != baseline["frame"]["slots"][0]
    })
    .await;
    assert_eq!(held["frame"]["slots"][1], 20);
    assert!(held["frame"]["slots"][0].as_u64().unwrap() < 220); // rejected batch never set full intensity
    let released = apply(
        &h,
        &session,
        serial,
        &manual["state"]["revision"],
        3,
        json!({"kind":"patch","changes":[edit("color-wheel",json!({"kind":"release"}))]}),
    )
    .await;
    serial += 1;
    assert_eq!(released["kind"], "applied");
    until(&h, |s| s["frame"]["slots"][1] == 40).await;
    fade_and_stop(&h, &session, serial, &released["state"]["revision"]).await;
    h.close().await;
}

async fn fade_and_stop(h: &Harness, session: &str, mut serial: u64, revision: &Value) {
    let zero = apply(
        h,
        session,
        serial,
        revision,
        1,
        json!({"kind":"level","value":0}),
    )
    .await;
    serial += 1;
    assert_eq!(zero["kind"], "applied");
    until(h, |s| {
        s["frame"]["slots"][0] == 0 && s["frame"]["slots"][1] == 40
    })
    .await;
    let stopped = apply(
        h,
        session,
        serial,
        &zero["state"]["revision"],
        2,
        json!({"kind":"stop"}),
    )
    .await;
    assert_eq!(stopped["state"]["sources"][0]["status"], "Running");
    assert_eq!(stopped["state"]["sources"][1]["status"], "Idle");
    until(h, |s| s["frame"]["slots"][1] == 20).await;
    assert!(!stopped.to_string().contains("lease"));
}
