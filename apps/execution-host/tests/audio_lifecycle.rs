mod support;
use serde_json::{Value, json};
use support::{
    audio::{applied, control},
    group::*,
    *,
};

#[test]
fn independent_audio_survives_controller_exit_and_original_resources_then_seeks_and_ends() {
    runtime().block_on(run());
}
async fn operate(
    h: &Harness,
    session: &str,
    serial: u64,
    state: &Value,
    operation: Value,
) -> Value {
    let accepted = control(h, session, serial, state, operation).await;
    assert_eq!(accepted["kind"], "accepted");
    assert_eq!(
        accepted["state"]["media"][0]["control"]["status"],
        "pending"
    );
    applied(h, &accepted).await
}
async fn run() {
    let mut h = Harness::prepared(|path| Some(audio::write(path)));
    let catalog = ok(h.get("/source")).await;
    assert_eq!(catalog["audio"]["output"], "software");
    let session = h.session().await;
    let acquired = h.acquire(&session, false).await;
    let started = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
    let running = operate(&h, &session, 3, &started["state"], json!({"kind":"play"})).await;
    assert_eq!(running["media"][0]["status"], "Following");
    assert_eq!(
        h.command(&session, 4, json!({"kind":"release"})).await["kind"],
        "released"
    );
    let stash = h.project.with_file_name("held.json");
    std::fs::rename(&h.project, &stash).unwrap();
    std::fs::remove_dir_all(format!("{}.assets", h.project.display())).unwrap();
    std::fs::remove_file(h.project.with_file_name("music.wav")).unwrap();
    let later = until(&h, |s| {
        s["state"]["media"][0]["positionMs"].as_u64().unwrap()
            > running["media"][0]["positionMs"].as_u64().unwrap() + 150
    })
    .await;
    assert!(later["state"]["owner"].is_null());
    assert_eq!(later["state"]["sources"][0]["status"], "Running");
    assert!(
        later["state"]["audio"]["frames"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    verify_pause_seek_end(&h).await;
    std::fs::rename(stash, &h.project).unwrap();
    h.close().await;
}
async fn verify_pause_seek_end(h: &Harness) {
    let other = h.session().await;
    let state = h.acquire(&other, false).await["state"].clone();
    let paused = operate(h, &other, 2, &state, json!({"kind":"pause"})).await;
    let position = paused["media"][0]["positionMs"].clone();
    tokio::time::sleep(std::time::Duration::from_millis(650)).await;
    let held = h.state().await;
    assert_eq!(held["media"][0]["positionMs"], position);
    assert_eq!(held["media"][0]["status"], "Paused");
    let seek = operate(
        h,
        &other,
        3,
        &held,
        json!({"kind":"seek","positionMs":3000,"playing":false}),
    )
    .await;
    assert_eq!(seek["media"][0]["positionMs"], 3000);
    assert_ne!(
        seek["media"][0]["generation"],
        held["media"][0]["generation"]
    );
    until(h, |s| s["frame"]["slots"][1] == 40).await;
    operate(
        h,
        &other,
        4,
        &seek,
        json!({"kind":"seek","positionMs":4800,"playing":true}),
    )
    .await;
    let ended = until(h, |s| {
        s["state"]["media"][0]["termination"]["reason"] == "ended"
            && s["state"]["media"][0]["termination"]["applied"] == true
    })
    .await;
    assert_eq!(ended["state"]["media"][0]["status"], "Stopped");
    assert_eq!(ended["state"]["audio"]["status"], "ended");
    assert_eq!(ended["state"]["sources"][0]["status"], "Running");
    until(h, |s| s["frame"]["slots"][1] == 20).await;
    let state = h.state().await;
    let restarted = operate(h, &other, 5, &state, json!({"kind":"play"})).await;
    assert!(restarted["media"][0]["positionMs"].as_u64().unwrap() < 1000);
    let stopped = operate(h, &other, 6, &restarted, json!({"kind":"stop"})).await;
    assert_eq!(stopped["media"][0]["status"], "Stopped");
    let stopped = until(h, |s| s["state"]["audio"]["status"] == "stopped").await;
    assert_eq!(stopped["state"]["audio"]["positionMs"], 0);
}
