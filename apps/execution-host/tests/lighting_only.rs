#![cfg(not(feature = "audio"))]
mod support;
use serde_json::{Value, json};
use std::{fs, process::Command};
use support::*;

#[test]
fn audio_content_is_explicitly_rejected_before_resource_preparation() {
    let dir = temporary();
    let project = dir.path().join("show.json");
    let manifest = group::write(&project);
    let original = fs::read(&project).unwrap();
    let valid: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    let audio = json!({"id":group::id(4),"priority":0,"selection":{"kind":"audioTimeline"}});
    for (index, value) in [
        json!({"version":2,"sources":[audio],"audio":{"output":"software"}}),
        json!({"version":2,"sources":valid["sources"],"audio":{"output":"systemDefault"}}),
        json!({"version":2,"sources":[audio]}),
    ]
    .iter()
    .enumerate()
    {
        fs::write(&manifest, serde_json::to_vec(value).unwrap()).unwrap();
        let run = dir.path().join(format!("run-{index}"));
        let error = rejection::rejected(
            Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"))
                .arg(&project)
                .arg("group")
                .arg(&manifest)
                .arg(&run)
                .arg("--software-output"),
            &run,
        );
        assert!(error.contains("未包含音频功能"), "{error}");
        assert!(!run.join("store/media").exists());
        assert_eq!(fs::read(&project).unwrap(), original);
    }
}

#[test]
fn audio_output_options_cannot_create_ownership_or_a_ready_process() {
    let dir = temporary();
    let project_path = dir.path().join("show.json");
    project(&project_path);
    let run = dir.path().join("run");
    let scope = dir.path().join("output");
    let error = rejection::rejected(
        Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"))
            .arg(project_path)
            .args(["sequence", SEQUENCE])
            .arg(&run)
            .arg("--software-output")
            .arg("--audio-scope")
            .arg(&scope),
        &run,
    );
    assert!(error.contains("未包含音频功能"), "{error}");
    assert!(!scope.exists());
    assert!(!run.exists());
}

#[test]
fn active_lighting_keeps_its_authority_when_unsupported_media_commands_arrive() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let source = ok(h.get("/source")).await;
        assert!(source["audio"].is_null());
        assert!(
            source["capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|v| { !v.as_str().unwrap().contains("Audio") })
        );
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let started = group::start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
        assert_eq!(started["kind"], "applied");
        let before = h.snapshot().await;
        let response = h
            .post(&format!("/sessions/{session}/commands"))
            .json(
                &json!({"serial":"3","ttlMs":1000,"command":{"kind":"submit",
                "expectedRevision":started["state"]["revision"],"action":{"kind":"media",
                "group":group::id(4),"generation":"1","action":{"kind":"play"}}}}),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 422);
        let after = group::until(&h, |s| s["cycles"] != before["cycles"]).await;
        assert_eq!(after["state"]["owner"], before["state"]["owner"]);
        assert_eq!(after["state"]["revision"], before["state"]["revision"]);
        assert_eq!(after["state"]["sources"][0]["status"], "Running");
        assert!(after["state"]["media"].is_null());
        h.close().await;
    });
}
