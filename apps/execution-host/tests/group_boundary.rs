mod support;
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Stdio},
};
use support::{group::*, *};

#[test]
fn invalid_manifests_never_publish_a_ready_process() {
    let dir = temporary();
    let project = dir.path().join("show.json");
    let manifest = write(&project);
    let valid: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    let mut variants = vec![
        json!({"version":2,"sources":valid["sources"]}),
        json!({"version":1,"sources":[]}),
        json!({"version":1,"sources":[valid["sources"][2]]}),
        json!({"version":1,"sources":[valid["sources"][0],valid["sources"][0]]}),
    ];
    variants.push(json!({"version":1,"sources":vec![valid["sources"][0].clone();65]}));
    let mut nil = valid.clone();
    nil["sources"][0]["id"] = json!("00000000-0000-0000-0000-000000000000");
    variants.push(nil);
    let mut wrong = valid.clone();
    wrong["sources"][0]["selection"]["id"] = json!(id(99));
    variants.push(wrong);
    let mut extra = valid.clone();
    extra["sources"][2]["selection"]["lease"] = json!("forged");
    variants.push(extra);
    let mut bytes: Vec<_> = variants
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap())
        .collect();
    bytes.push(vec![b' '; 65_537]);
    for (i, bytes) in bytes.iter().enumerate() {
        fs::write(&manifest, bytes).unwrap();
        let run = dir.path().join(format!("run-{i}"));
        let output = Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"))
            .arg(&project)
            .arg("group")
            .arg(&manifest)
            .arg(&run)
            .arg("--software-output")
            .stdout(Stdio::null())
            .output()
            .unwrap();
        assert!(!output.status.success(), "错误来源清单 {i} 被接纳");
        assert!(!run.join("discovery.json").exists());
    }
}
#[test]
fn group_profile_preserves_permissions_rejects_foreign_operations_and_retries_lost_replies() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::start_group();
    let read = h.discovery["readToken"].as_str().unwrap();
    assert_eq!(
        h.http
            .post(h.url("/sessions"))
            .bearer_auth(read)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        h.get("/source")
            .header("origin", "http://example.test")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let id = h.session().await;
    let acquired = h.acquire(&id, false).await;
    let foreign=h.command(&id,2,json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":{"kind":"stop"}})).await;
    assert_eq!(foreign["kind"], "rejected");
    assert_eq!(h.state().await["revision"], acquired["state"]["revision"]);
    let source = ok(h.get("/source")).await;
    let start = json!({"kind":"submit","expectedRevision":acquired["state"]["revision"],"action":action(1,json!({"kind":"start","step":source["sources"][0]["steps"][0]["id"]}))});
    let response = h
        .post(&format!("/sessions/{id}/commands"))
        .json(&json!({"serial":"3","ttlMs":5000,"command":start}))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    drop(response);
    let result = h.command(&id, 3, start.clone()).await;
    assert_eq!(result["kind"], "applied");
    let sample = h.snapshot().await;
    let later = until(&h, |s| {
        s["frame"]["slots"][0] != sample["frame"]["slots"][0]
    })
    .await;
    assert_eq!(h.command(&id, 3, start).await, result);
    assert_eq!(later["state"]["revision"], result["state"]["revision"]);
    let wrong = json!({"serial":"4","ttlMs":5000,"command":{"kind":"submit","expectedRevision":result["state"]["revision"],"action":{
        "kind":"source","source":group::id(1),"action":{"kind":"stop","key":0}}}});
    assert_eq!(
        h.post(&format!("/sessions/{id}/commands"))
            .json(&wrong)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(h.state().await["sources"][0]["status"], "Running");
    h.close().await;
}
