#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use support::{
    audio::{applied, control},
    group::until,
    *,
};

#[test]
fn fractional_frame_seek_uses_actual_quantized_source_position_and_can_resume() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write_rate(p, 44100)));
        let session = h.session().await;
        let acquired = h.acquire(&session, false).await;
        let accepted = control(
            &h,
            &session,
            2,
            &acquired["state"],
            json!({"kind":"seek","positionMs":2501,"playing":false}),
        )
        .await;
        assert_eq!(accepted["kind"], "accepted");
        let paused = applied(&h, &accepted).await;
        // Native seeking floors to sample frames; 2501 ms is represented by frame 110294.
        assert_eq!(paused["media"][0]["positionMs"], 2500);
        let native = until(&h, |s| s["state"]["audio"]["status"] == "paused").await;
        assert_eq!(native["state"]["audio"]["positionMs"], 2500);
        let accepted = control(&h, &session, 3, &h.state().await, json!({"kind":"play"})).await;
        applied(&h, &accepted).await;
        until(&h, |s| {
            s["state"]["media"][0]["positionMs"]
                .as_u64()
                .is_some_and(|v| v > 2600)
        })
        .await;
        h.close().await;
    });
}
