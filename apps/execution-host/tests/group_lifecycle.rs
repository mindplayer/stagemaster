mod support;
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::{group::*, *};

#[test]
fn isolated_group_controller_helper() {
    let Some(path) = std::env::var_os("STAGEMASTER_GROUP_TEST_DISCOVERY") else {
        return;
    };
    runtime().block_on(async {
        let discovery:Value=serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let http=client();
        let base=discovery["url"].as_str().unwrap();
        let token=discovery["controlToken"].as_str().unwrap();
        let session=ok(http.post(format!("{base}/sessions")).bearer_auth(token)).await["sessionId"].as_str().unwrap().to_owned();
        let acquired=command(&http,&discovery,&session,1,json!({"kind":"acquire","durationMs":60000,"takeover":false})).await;
        let directory=ok(http.get(format!("{base}/source")).bearer_auth(token)).await;
        let mut state=acquired["state"].clone();
        for i in 0..2 {
            let source=&directory["sources"][i];
            let result=command(&http,&discovery,&session,u64::try_from(i+2).unwrap(),json!({"kind":"submit","expectedRevision":state["revision"],
                "action":{"kind":"source","source":source["id"],"action":{"kind":"start","step":source["steps"][0]["id"]}}})).await;
            assert_eq!(result["kind"],"applied"); state=result["state"].clone();
        }
        fs::write(std::env::var_os("STAGEMASTER_GROUP_TEST_READY").unwrap(),serde_json::to_vec(&json!({"sessionId":session,"state":state})).unwrap()).unwrap();
    });
    loop {
        std::thread::park();
    }
}
#[test]
fn multi_source_execution_survives_controller_death_and_missing_preparation_files() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::start_group();
    let directory = ok(h.get("/source")).await;
    let ready = h.directory.path().join("group-ready.json");
    let error = h.directory.path().join("controller.log");
    let mut controller = Process(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "isolated_group_controller_helper", "--nocapture"])
            .env(
                "STAGEMASTER_GROUP_TEST_DISCOVERY",
                h.directory.path().join("run/discovery.json"),
            )
            .env("STAGEMASTER_GROUP_TEST_READY", &ready)
            .stdout(Stdio::null())
            .stderr(fs::File::create(&error).unwrap())
            .spawn()
            .unwrap(),
    );
    let end = Instant::now() + Duration::from_secs(10);
    while !ready.exists() {
        assert!(
            controller.0.try_wait().unwrap().is_none(),
            "{}",
            fs::read_to_string(&error).unwrap()
        );
        assert!(Instant::now() < end);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let original: Value = serde_json::from_slice(&fs::read(&ready).unwrap()).unwrap();
    let before = h.snapshot().await;
    controller.0.kill().unwrap();
    controller.0.wait().unwrap();
    fs::remove_file(&h.project).unwrap();
    fs::remove_file(h.directory.path().join("sources.json")).unwrap();
    let after = until(&h, |s| {
        s["frame"]["slots"][0] != before["frame"]["slots"][0]
    })
    .await;
    assert!(h.process.0.try_wait().unwrap().is_none());
    assert_eq!(after["state"]["boot"], before["state"]["boot"]);
    assert_eq!(after["state"]["layout"], before["state"]["layout"]);
    assert_eq!(after["state"]["revision"], original["state"]["revision"]);
    assert_eq!(after["frame"]["slots"][1], 40);
    assert!(
        after["state"]["sources"].as_array().unwrap()[..2]
            .iter()
            .all(|s| s["status"] == "Running")
    );
    assert_eq!(ok(h.get("/source")).await, directory);
    let session = h.session().await;
    assert_eq!(h.acquire(&session, false).await["kind"], "rejected");
    let acquired = h
        .command(
            &session,
            2,
            json!({"kind":"acquire","durationMs":60000,"takeover":true}),
        )
        .await;
    let old = apply(
        &h,
        original["sessionId"].as_str().unwrap(),
        4,
        &acquired["state"]["revision"],
        1,
        json!({"kind":"stop"}),
    )
    .await;
    assert_eq!(old["code"], "Runtime(Lease)");
    let paused = apply(
        &h,
        &session,
        3,
        &acquired["state"]["revision"],
        1,
        json!({"kind":"pause"}),
    )
    .await;
    assert_eq!(paused["state"]["sources"][0]["status"], "Paused");
    assert_eq!(paused["state"]["sources"][1]["status"], "Running");
    fs::write(&h.project, &h.original).unwrap();
    h.close().await;
}
