mod support;
use serde_json::{Value, json};
use stagemaster_audio::OutputScope;
use std::{fs, process::Command};
use support::{audio, rejection::rejected, temporary};

#[test]
fn direct_background_launch_cannot_bypass_a_busy_output_route() {
    let dir = temporary();
    let project = dir.path().join("show.json");
    let manifest = audio::write(&project);
    let scope_path = dir.path().join("output");
    let scope = OutputScope::new(scope_path.clone()).unwrap();
    let _lease = scope.reserve().unwrap();
    let run = dir.path().join("blocked");
    let mut command = Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"));
    command
        .arg(project)
        .arg("group")
        .arg(manifest)
        .arg(&run)
        .arg("--software-output")
        .arg("--audio-scope")
        .arg(scope_path);
    assert!(rejected(&mut command, &run).contains("占用"));
    assert!(!run.join("store/media").exists());
    assert!(scope.reserve().is_err());
}

#[test]
fn system_audio_without_an_explicit_scope_is_rejected_before_device_access() {
    let dir = temporary();
    let project = dir.path().join("show.json");
    let manifest = audio::write(&project);
    let mut value: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    value["audio"] = json!({"output":"systemDefault"});
    fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    let run = dir.path().join("blocked");
    let mut command = Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"));
    command
        .arg(project)
        .arg("group")
        .arg(manifest)
        .arg(&run)
        .arg("--software-output");
    assert!(rejected(&mut command, &run).contains("必须指定"));
    assert!(!run.join("store/media").exists());
}
