mod support;
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::{group::until, *};

#[test]
fn isolated_audio_controller_helper() {
    let Some(path) = std::env::var_os("STAGEMASTER_AUDIO_TEST_DISCOVERY") else {
        return;
    };
    runtime().block_on(async {
        let discovery: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let http = client();
        let base = discovery["url"].as_str().unwrap();
        let token = discovery["controlToken"].as_str().unwrap();
        let session =
            ok(http.post(format!("{base}/sessions")).bearer_auth(token)).await["sessionId"]
                .as_str()
                .unwrap()
                .to_owned();
        let acquired = command(
            &http,
            &discovery,
            &session,
            1,
            json!({"kind":"acquire","durationMs":60000,"takeover":false}),
        )
        .await;
        let accepted = command(&http, &discovery, &session, 2, json!({
            "kind":"submit","expectedRevision":acquired["state"]["revision"],
            "action":{"kind":"media","group":group::id(2),
                "generation":acquired["state"]["media"][0]["generation"],"action":{"kind":"play"}}
        })).await;
        assert_eq!(accepted["kind"], "accepted");
        fs::write(
            std::env::var_os("STAGEMASTER_AUDIO_TEST_READY").unwrap(),
            serde_json::to_vec(&accepted).unwrap(),
        )
        .unwrap();
    });
    loop {
        std::thread::park();
    }
}

#[test]
fn actual_audio_process_survives_a_killed_controller_without_new_owner_or_source() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::prepared(|p| Some(audio::write(p)));
    let ready = h.directory.path().join("audio-ready.json");
    let error = h.directory.path().join("controller.log");
    let mut controller = Process(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "isolated_audio_controller_helper", "--nocapture"])
            .env(
                "STAGEMASTER_AUDIO_TEST_DISCOVERY",
                h.directory.path().join("run/discovery.json"),
            )
            .env("STAGEMASTER_AUDIO_TEST_READY", &ready)
            .stdout(Stdio::null())
            .stderr(fs::File::create(&error).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready.exists() {
        assert!(
            controller.0.try_wait().unwrap().is_none(),
            "{}",
            fs::read_to_string(&error).unwrap()
        );
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let accepted: Value = serde_json::from_slice(&fs::read(ready).unwrap()).unwrap();
    let playing = audio::applied(&h, &accepted).await;
    controller.0.kill().unwrap();
    assert!(!controller.0.wait().unwrap().success());
    let later = until(&h, |s| {
        s["state"]["media"][0]["positionMs"].as_u64().unwrap()
            > playing["media"][0]["positionMs"].as_u64().unwrap() + 200
    })
    .await;
    assert_eq!(later["state"]["boot"], playing["boot"]);
    assert_eq!(later["state"]["owner"], playing["owner"]);
    assert_eq!(
        later["state"]["audio"]["instance"],
        playing["audio"]["instance"]
    );
    assert_eq!(later["state"]["media"][0]["status"], "Following");
    assert!(
        later["state"]["audio"]["frames"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > playing["audio"]["frames"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
    );
    let other = h.session().await;
    let takeover = h.acquire(&other, true).await;
    let accepted = audio::control(&h, &other, 2, &takeover["state"], json!({"kind":"pause"})).await;
    assert_eq!(
        audio::applied(&h, &accepted).await["media"][0]["status"],
        "Paused"
    );
    h.close().await;
}
