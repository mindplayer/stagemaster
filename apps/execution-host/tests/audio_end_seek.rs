#![cfg(feature = "audio")]
mod support;
use serde_json::{Value, json};
use support::{
    audio::{applied, control},
    group::*,
    *,
};

async fn operate(h: &Harness, session: &str, serial: u64, state: &Value, op: Value) -> Value {
    let accepted = control(h, session, serial, state, op).await;
    assert_eq!(accepted["kind"], "accepted", "{accepted}");
    applied(h, &accepted).await
}
fn end(playing: bool) -> Value {
    json!({"kind":"seek", "positionMs":5000, "playing":playing})
}
async fn stable_end(h: &Harness, state: &Value) {
    assert_eq!(state["media"][0]["status"], "Stopped");
    let ended = until(h, |s| s["state"]["audio"]["status"] == "ended").await;
    assert_eq!(ended["state"]["audio"]["positionMs"], 5000);
    assert!(ended["state"]["audio"]["problem"].is_null());
    let generation = state["media"][0]["generation"].clone();
    tokio::time::sleep(std::time::Duration::from_millis(650)).await;
    let later = h.state().await;
    assert_eq!(later["audio"]["status"], "ended");
    assert_eq!(later["audio"]["positionMs"], 5000);
    assert_eq!(later["audio"]["frames"], ended["state"]["audio"]["frames"]);
    assert_eq!(later["media"][0]["generation"], generation);
    assert!(later["media"][0]["termination"].is_null());
}

#[test]
fn exact_end_from_ready_running_and_paused_releases_only_music_and_can_replay() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let idle = until(&h, |s| {
            s["frame"]["slots"]
                .as_array()
                .is_some_and(|v| v.len() == 512)
        })
        .await["frame"]["slots"]
            .clone();
        assert_eq!(ok(h.get("/source")).await["audio"]["seekIncludesEnd"], true);
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let ready_end = operate(&h, &session, 2, &acquired["state"], end(true)).await;
        stable_end(&h, &ready_end).await;
        assert_eq!(h.snapshot().await["frame"]["slots"], idle);
        let running = operate(&h, &session, 3, &ready_end, json!({"kind":"play"})).await;
        assert!(running["audio"]["positionMs"].as_u64().unwrap() < 1000);
        until(&h, |s| {
            s["state"]["audio"]["frames"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > 0
        })
        .await;
        let state = h.state().await;
        let playing_end = operate(&h, &session, 4, &state, end(true)).await;
        stable_end(&h, &playing_end).await;
        let autonomous = start(&h, &session, 5, &playing_end["revision"], 0).await;
        assert_eq!(autonomous["kind"], "applied");
        let paused = operate(
            &h,
            &session,
            6,
            &autonomous["state"],
            json!({"kind":"seek", "positionMs":3000, "playing":false}),
        )
        .await;
        assert_eq!(paused["media"][0]["status"], "Paused");
        until(&h, |s| s["frame"]["slots"][1] == 40).await;
        let paused_end = operate(&h, &session, 7, &paused, end(false)).await;
        stable_end(&h, &paused_end).await;
        let retained = until(&h, |s| s["frame"]["slots"][1] == 20).await;
        assert_eq!(retained["state"]["sources"][0]["status"], "Running");
        // Repeat the same accepted command: the same receipt, no second generation or release.
        let duplicate = control(&h, &session, 7, &paused, end(false)).await;
        assert_eq!(duplicate["kind"], "accepted");
        assert_eq!(
            h.state().await["media"][0]["generation"],
            paused_end["media"][0]["generation"]
        );
        let repeated = operate(&h, &session, 8, &paused_end, end(false)).await;
        stable_end(&h, &repeated).await;
        verify_refusal_and_replay(&h, &session, &repeated, &paused).await;
        h.close().await;
    });
}

async fn verify_refusal_and_replay(h: &Harness, session: &str, ended: &Value, old: &Value) {
    let before = h.state().await;
    let rejected = control(
        h,
        session,
        9,
        &before,
        json!({"kind":"seek", "positionMs":5001, "playing":false}),
    )
    .await;
    assert_eq!(rejected["kind"], "rejected");
    let mut outdated = h.state().await;
    outdated["media"][0]["generation"] = old["media"][0]["generation"].clone();
    assert_eq!(
        control(h, session, 10, &outdated, end(true)).await["kind"],
        "rejected"
    );
    assert_eq!(
        h.state().await["media"][0]["generation"],
        ended["media"][0]["generation"]
    );
    let current = h.state().await;
    let restarted = operate(h, session, 11, &current, json!({"kind":"play"})).await;
    assert_eq!(restarted["media"][0]["status"], "Following");
    assert!(restarted["audio"]["positionMs"].as_u64().unwrap() < 1000);
    assert_ne!(restarted["audio"]["instance"], ended["audio"]["instance"]);
    let stopped = operate(h, session, 12, &restarted, json!({"kind":"stop"})).await;
    assert_eq!(stopped["media"][0]["status"], "Stopped");
    let state = until(h, |s| s["state"]["audio"]["status"] == "stopped").await;
    assert_eq!(state["state"]["audio"]["positionMs"], 0);
}
