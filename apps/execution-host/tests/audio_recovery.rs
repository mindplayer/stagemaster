mod support;
use serde_json::{Value, json};
use std::fs;
use support::{
    audio::{applied, control},
    group::*,
    *,
};

async fn operate(h: &Harness, who: &str, serial: u64, state: &Value, action: Value) -> Value {
    let result = control(h, who, serial, state, action).await;
    assert_eq!(result["kind"], "accepted", "{result}");
    applied(h, &result).await
}

#[test]
fn failed_provider_rebuilds_on_same_host_keeps_other_lights_and_waits_for_explicit_play() {
    runtime().block_on(recover());
}
async fn recover() {
    let mut h = Harness::prepared_scoped(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
    let scope = stagemaster_audio::OutputScope::new(h.directory.path().join("output")).unwrap();
    assert!(scope.reserve().is_err());
    assert_eq!(
        ok(h.get("/source")).await["audio"]["providerRecovery"],
        true
    );
    let who = h.session().await;
    let acquired = h.acquire(&who, false).await;
    let independent = start(&h, &who, 2, &acquired["state"]["revision"], 0).await;
    let playing = operate(&h, &who, 3, &independent["state"], json!({"kind":"play"})).await;
    let copy = fs::read_dir(h.directory.path().join("run/store/media"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let healthy = fs::read(&copy).unwrap();
    fs::write(&copy, b"owned test source damaged").unwrap();
    let rejected = control(
        &h,
        &who,
        4,
        &playing,
        json!({"kind":"seek","positionMs":2500,"playing":true}),
    )
    .await;
    assert_eq!(rejected["kind"], "accepted");
    let failed = until(&h, |s| {
        s["state"]["audio"]["status"] == "failed"
            && s["state"]["media"][0]["control"]["status"] == "failed"
            && s["state"]["media"][0]["status"] == "Stopped"
    })
    .await["state"]
        .clone();
    assert_eq!(failed["sources"][0]["status"], "Running");
    assert!(scope.reserve().is_err());
    let recovery = json!({"kind":"recover","positionMs":2500});
    // A decodable WAV with changed PCM is still the wrong immutable resource.
    let mut substituted = healthy.clone();
    substituted[44] ^= 1;
    fs::write(&copy, substituted).unwrap();
    let attempt = control(&h, &who, 5, &failed, recovery.clone()).await;
    assert_eq!(attempt["kind"], "accepted");
    let still_failed = until(&h, |s| {
        s["state"]["audio"]["status"] == "failed"
            && s["state"]["media"][0]["control"]["status"] == "failed"
            && s["state"]["media"][0]["control"]["request"]
                == attempt["state"]["media"][0]["control"]["request"]
    })
    .await["state"]
        .clone();
    assert_eq!(
        still_failed["media"][0]["generation"],
        failed["media"][0]["generation"]
    );
    assert!(
        still_failed["audio"]["problem"]
            .as_str()
            .unwrap()
            .contains("后台音乐资源校验失败")
    );
    fs::write(&copy, healthy).unwrap();
    // Recovery uses the fixed private snapshot even after the editable project disappears.
    fs::remove_file(&h.project).unwrap();
    let prepared = operate(&h, &who, 6, &still_failed, recovery.clone()).await;
    assert!(scope.reserve().is_err());
    verify_recovery(&h, &who, &still_failed, &prepared, recovery).await;
    assert!(!h.project.exists());
    // Restore only after recovery/replay checks; the common shutdown assertion verifies the file.
    fs::write(&h.project, &h.original).unwrap();
    h.close().await;
    assert!(scope.reserve().is_ok());
}
async fn verify_recovery(
    h: &Harness,
    who: &str,
    failed: &Value,
    prepared: &Value,
    recovery: Value,
) {
    let paused = until(h, |s| s["state"]["audio"]["status"] == "paused").await["state"].clone();
    assert_eq!(paused["audio"]["positionMs"], 2500);
    assert_eq!(paused["audio"]["loopState"]["pass"], "1");
    assert_ne!(paused["audio"]["instance"], failed["audio"]["instance"]);
    assert_ne!(
        paused["media"][0]["generation"],
        failed["media"][0]["generation"]
    );
    assert_eq!(paused["sources"][0]["status"], "Running");
    assert_eq!(paused["owner"], failed["owner"]);
    assert_eq!(paused["fault"], false);
    assert!(paused["audio"]["problem"].is_null());
    let before = h.snapshot().await;
    tokio::time::sleep(std::time::Duration::from_millis(140)).await;
    let later = h.snapshot().await;
    assert_eq!(later["state"]["audio"]["positionMs"], 2500);
    assert_eq!(later["state"]["audio"]["frames"], "0");
    assert_ne!(later["frame"]["slots"][0], before["frame"]["slots"][0]);
    let duplicate = control(h, who, 6, failed, recovery).await;
    assert_eq!(duplicate["kind"], "accepted");
    assert_eq!(
        h.state().await["audio"]["instance"],
        paused["audio"]["instance"]
    );
    let mut stale = prepared.clone();
    stale["media"][0]["generation"] = failed["media"][0]["generation"].clone();
    assert_eq!(
        control(h, who, 7, &stale, json!({"kind":"recover","positionMs":0})).await["kind"],
        "rejected"
    );
    let playing = operate(h, who, 8, &h.state().await, json!({"kind":"play"})).await;
    assert_eq!(playing["media"][0]["status"], "Following");
    until(h, |s| s["state"]["audio"]["loopState"]["pass"] == "2").await;
    assert!(h.get("/state").send().await.unwrap().status().is_success());
}
