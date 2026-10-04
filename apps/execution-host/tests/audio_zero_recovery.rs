#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use std::fs;
use support::{
    audio::{applied, control},
    group::until,
    *,
};

#[test]
fn explicit_zero_recovery_uses_a_new_healthy_paused_voice_and_original_completion() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let who = h.session().await;
        let acquired = h.acquire(&who, false).await;
        let play = control(&h, &who, 2, &acquired["state"], json!({"kind":"play"})).await;
        assert_eq!(play["kind"], "accepted");
        let playing = applied(&h, &play).await;
        let copy = fs::read_dir(h.directory.path().join("run/store/media"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let original = fs::read(&copy).unwrap();
        fs::write(&copy, b"owned private copy fault injection").unwrap();
        let seek = control(
            &h,
            &who,
            3,
            &playing,
            json!({"kind":"seek","positionMs":3000,"playing":false}),
        )
        .await;
        assert_eq!(seek["kind"], "accepted");
        let failed = until(&h, |v| {
            v["state"]["media"][0]["control"]["status"] == "failed"
                && v["state"]["media"][0]["control"]["request"]
                    == seek["state"]["media"][0]["control"]["request"]
                && v["state"]["audio"]["status"] == "failed"
        })
        .await["state"]
            .clone();
        fs::write(&copy, original).unwrap();
        let recovery = control(
            &h,
            &who,
            4,
            &failed,
            json!({"kind":"recover","positionMs":0}),
        )
        .await;
        assert_eq!(recovery["kind"], "accepted");
        assert_eq!(
            recovery["state"]["media"][0]["control"]["status"],
            "pending"
        );
        let request = recovery["state"]["media"][0]["control"]["request"].clone();
        let paused = applied(&h, &recovery).await;
        assert_eq!(paused["media"][0]["control"]["request"], request);
        assert_eq!(paused["media"][0]["status"], "Paused");
        assert_eq!(paused["audio"]["positionMs"], 0);
        assert_eq!(paused["audio"]["frames"], "0");
        assert!(paused["audio"]["problem"].is_null());
        assert_ne!(paused["audio"]["instance"], failed["audio"]["instance"]);
        assert_ne!(
            paused["media"][0]["generation"],
            failed["media"][0]["generation"]
        );
        assert_eq!(paused["owner"], failed["owner"]);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let held = h.state().await;
        assert_eq!(held["audio"]["positionMs"], 0);
        assert_eq!(held["audio"]["frames"], "0");
        let resume = control(&h, &who, 5, &held, json!({"kind":"play"})).await;
        assert_eq!(resume["kind"], "accepted");
        let resumed = applied(&h, &resume).await;
        assert_eq!(resumed["media"][0]["status"], "Following");
        assert_eq!(resumed["audio"]["instance"], held["audio"]["instance"]);
        h.close().await;
    });
}
