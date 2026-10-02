mod support;
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::*;

#[test]
fn isolated_controller_helper() {
    let Some(path) = std::env::var_os("STAGEMASTER_HOST_TEST_DISCOVERY") else {
        return;
    };
    runtime().block_on(run_controller(path));
    loop {
        std::thread::park();
    }
}

async fn run_controller(path: std::ffi::OsString) {
    let discovery: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let http = client();
    let base = discovery["url"].as_str().unwrap();
    let token = discovery["controlToken"].as_str().unwrap();
    let session = ok(http.post(format!("{base}/sessions")).bearer_auth(token)).await["sessionId"]
        .as_str()
        .unwrap()
        .to_owned();
    let acquired = command(
        &http,
        &discovery,
        &session,
        1,
        json!({"kind":"acquire","durationMs":60_000,"takeover":false}),
    )
    .await;
    let step =
        ok(http.get(format!("{base}/source")).bearer_auth(token)).await["steps"][0]["id"].clone();
    let started=command(&http,&discovery,&session,2,json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":{"kind":"start","step":step}})).await;
    assert_eq!(started["kind"], "applied");
    fs::write(
        std::env::var_os("STAGEMASTER_HOST_TEST_READY").unwrap(),
        serde_json::to_vec(&json!({"sessionId":session,"state":started["state"]})).unwrap(),
    )
    .unwrap();
}

#[test]
fn killing_a_real_controller_keeps_the_same_show_running_and_allows_takeover() {
    runtime().block_on(
        killing_a_real_controller_keeps_the_same_show_running_and_allows_takeover_async(),
    );
}
async fn killing_a_real_controller_keeps_the_same_show_running_and_allows_takeover_async() {
    let mut h = Harness::start();
    let ready = h.directory.path().join("controller-ready.json");
    let mut controller = Process(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "isolated_controller_helper", "--nocapture"])
            .env(
                "STAGEMASTER_HOST_TEST_DISCOVERY",
                h.directory.path().join("run/discovery.json"),
            )
            .env("STAGEMASTER_HOST_TEST_READY", &ready)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready.exists() {
        assert!(
            controller.0.try_wait().unwrap().is_none(),
            "独立控制客户端提前退出"
        );
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let original: Value = serde_json::from_slice(&fs::read(&ready).unwrap()).unwrap();
    let before = h.state().await;
    controller.0.kill().unwrap();
    controller.0.wait().unwrap();
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(h.process.0.try_wait().unwrap().is_none());
    let after = h.state().await;
    assert_eq!(after["instance"], original["state"]["instance"]);
    assert_eq!(after["status"], "Running");
    assert!(
        after["elapsedMs"].as_str().unwrap().parse::<u64>().unwrap()
            > before["elapsedMs"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
    );
    let second = h.session().await;
    let refused = h.acquire(&second, false).await;
    assert_eq!(refused["kind"], "rejected");
    let acquired = h
        .command(
            &second,
            2,
            json!({"kind":"acquire","durationMs":60_000,"takeover":true}),
        )
        .await;
    assert_eq!(acquired["state"]["instance"], after["instance"]);
    let late=h.command(original["sessionId"].as_str().unwrap(),3,json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":{"kind":"stop"}})).await;
    assert_eq!(late["code"], "Runtime(Lease)");
    assert_eq!(h.state().await["status"], "Running");
    let pause=h.command(&second,3,json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":{"kind":"pause"}})).await;
    assert_eq!(pause["kind"], "applied");
    assert_eq!(pause["state"]["status"], "Paused");
    h.close().await;
}

#[test]
fn preparation_is_idle_retries_are_idempotent_and_business_refusals_consume_serials() {
    runtime().block_on(
        preparation_is_idle_retries_are_idempotent_and_business_refusals_consume_serials_async(),
    );
}
async fn preparation_is_idle_retries_are_idempotent_and_business_refusals_consume_serials_async() {
    let mut h = Harness::start();
    let initial = h.state().await;
    assert_eq!(initial["status"], "Idle");
    assert!(initial["instance"].is_null());
    assert!(initial["owner"].is_null());
    let source = ok(h.get("/source")).await;
    assert_eq!(source["selection"]["id"], SEQUENCE);
    assert_eq!(source["physicalOutput"], false);
    let session = h.session().await;
    assert!(h.state().await["owner"].is_null());
    let acquired = h.acquire(&session, false).await;
    let start = h
        .start_program(&session, &acquired["state"]["revision"], 2)
        .await;
    let repeat = h
        .start_program(&session, &acquired["state"]["revision"], 2)
        .await;
    assert_eq!(start, repeat);
    let refused = h
        .command(
            &session,
            3,
            json!({"kind":"submit","expectedRevision":"0","action":{"kind":"pause"}}),
        )
        .await;
    assert_eq!(refused["code"], "Revision");
    let paused=h.command(&session,4,json!({"kind":"submit","expectedRevision":refused["state"]["revision"],"action":{"kind":"pause"}})).await;
    assert_eq!(paused["kind"], "applied");
    assert_eq!(paused["state"]["instance"], start["state"]["instance"]);
    assert!(!serde_json::to_string(&paused).unwrap().contains("lease"));
    let release = h.command(&session, 5, json!({"kind":"release"})).await;
    assert_eq!(release["kind"], "released");
    assert_eq!(h.state().await["status"], "Paused");
    h.close().await;
}
