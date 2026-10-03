mod support;
use serde_json::{Value, json};
use support::{
    audio::{applied, control},
    group::*,
    *,
};

async fn operate(h: &Harness, session: &str, serial: u64, op: Value) -> Value {
    let accepted = control(h, session, serial, &h.state().await, op).await;
    assert_eq!(accepted["kind"], "accepted", "{accepted}");
    applied(h, &accepted).await
}
#[test]
fn actual_pcm_finite_repeats_keep_one_instance_and_light_source_then_finish() {
    runtime().block_on(async {
        let mut h =
            Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"count","count":2}))));
        let catalog = ok(h.get("/source")).await;
        assert_eq!(catalog["audio"]["performanceLoops"], true);
        let session = h.session().await;
        h.acquire(&session, false).await;
        let running = operate(
            &h,
            &session,
            2,
            json!({"kind":"seek","positionMs":2400,"playing":true}),
        )
        .await;
        let blue = until(&h, |s| {
            s["state"]["audio"]["loopState"]["pass"] == "1" && s["frame"]["slots"][1] == 40
        })
        .await;
        let red = until(&h, |s| {
            s["state"]["audio"]["loopState"]["pass"] == "2" && s["frame"]["slots"][1] == 20
        })
        .await;
        assert_eq!(
            red["state"]["audio"]["instance"],
            running["audio"]["instance"]
        );
        assert_eq!(
            red["state"]["media"][0]["generation"],
            running["media"][0]["generation"]
        );
        assert!(
            red["state"]["audio"]["positionMs"].as_u64()
                < blue["state"]["audio"]["positionMs"].as_u64()
        );
        assert!(
            red["state"]["audio"]["frames"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > blue["state"]["audio"]["frames"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
        );
        let ended = until(&h, |s| {
            s["state"]["audio"]["status"] == "ended"
                && s["state"]["media"][0]["status"] == "Stopped"
        })
        .await;
        assert!(ended["state"]["audio"]["problem"].is_null());
        assert!(ended["state"]["audio"]["loopState"].is_null());
        let replay = operate(
            &h,
            &session,
            3,
            json!({"kind":"seek","positionMs":2500,"playing":false}),
        )
        .await;
        assert_ne!(replay["audio"]["instance"], running["audio"]["instance"]);
        assert_eq!(replay["audio"]["loopState"]["pass"], "1");
        let ended = operate(
            &h,
            &session,
            4,
            json!({"kind":"seek","positionMs":5000,"playing":true}),
        )
        .await;
        assert_eq!(ended["media"][0]["status"], "Stopped");
        h.close().await;
    });
}

#[test]
fn paused_exit_cancellation_stale_targets_and_running_exit_use_actual_callback_receipts() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let session = h.session().await;
        h.acquire(&session, false).await;
        let paused = operate(
            &h,
            &session,
            2,
            json!({"kind":"seek","positionMs":2500,"playing":false}),
        )
        .await;
        let intent = loops::exit(&paused, true);
        let exiting = operate(&h, &session, 3, intent.clone()).await;
        until(&h, |s| {
            s["state"]["audio"]["loopState"]["exitRequested"] == true
        })
        .await;
        assert_eq!(exiting["audio"]["frames"], "0");
        let cancelled = operate(&h, &session, 4, loops::exit(&paused, false)).await;
        let observed = until(&h, |s| {
            s["state"]["audio"]["loopState"]["exitRequested"] == false
                && s["state"]["audio"]["loopState"]["pendingExit"].is_null()
        })
        .await;
        assert_eq!(observed["state"]["audio"]["positionMs"], 2500);
        assert_eq!(cancelled["media"][0]["status"], "Paused");
        operate(&h, &session, 5, json!({"kind":"play"})).await;
        until(&h, |s| s["state"]["audio"]["loopState"]["pass"] == "2").await;
        let next = operate(&h, &session, 6, json!({"kind":"pause"})).await;
        assert_eq!(next["audio"]["instance"], paused["audio"]["instance"]);
        let refused = control(&h, &session, 7, &h.state().await, intent).await;
        assert_eq!(refused["kind"], "rejected");
        let intact = h.state().await;
        assert_eq!(intact["media"][0]["status"], "Paused");
        assert!(intact["audio"]["problem"].is_null());
        assert_eq!(intact["audio"]["loopState"]["exitRequested"], false);
        operate(&h, &session, 8, loops::exit(&intact, true)).await;
        operate(&h, &session, 9, json!({"kind":"play"})).await;
        until(&h, |s| {
            s["state"]["audio"]["positionMs"]
                .as_u64()
                .is_some_and(|v| v > 3100)
                && s["state"]["audio"]["loopState"].is_null()
        })
        .await;
        let located = operate(
            &h,
            &session,
            10,
            json!({"kind":"seek","positionMs":2500,"playing":false}),
        )
        .await;
        assert_ne!(located["audio"]["instance"], paused["audio"]["instance"]);
        let refused = control(
            &h,
            &session,
            11,
            &h.state().await,
            loops::exit(&paused, true),
        )
        .await;
        assert_eq!(refused["kind"], "rejected");
        operate(&h, &session, 12, json!({"kind":"stop"})).await;
        until(&h, |s| s["state"]["audio"]["loopState"].is_null()).await;
        h.close().await;
    });
}
