#![cfg(feature = "audio")]
mod support;
use serde_json::{Value, json};
use std::{fs, process::Command};
use support::{
    audio::{applied, control},
    group::*,
    *,
};

#[test]
fn preparation_requires_valid_explicit_audio_and_untampered_portable_assets() {
    for case in 0..6 {
        let dir = temporary();
        let show = dir.path().join("show.json");
        let manifest = audio::write(&show);
        let mut value: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        match case {
            0 => {
                fs::remove_dir_all(format!("{}.assets", show.display())).unwrap();
            }
            1 => {
                let asset = fs::read_dir(format!("{}.assets", show.display()))
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                fs::write(asset, b"damaged original resource").unwrap();
            }
            2 => value["version"] = 1.into(),
            3 => value["audio"] = json!({"output":"unknown"}),
            4 => {
                value.as_object_mut().unwrap().remove("audio");
            }
            _ => {
                let mut raw: Value = serde_json::from_slice(&fs::read(&show).unwrap()).unwrap();
                raw["requires"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"key":"media.audio-loop-regions","version":1}));
                raw["media"]["audioEditing"]["loopRegions"] = json!([{"id":id(99),"name":"重复","startMs":0,"endMs":1000,"plays":{"kind":"count","count":0},"enabled":true,"locked":false}]);
                fs::write(&show, serde_json::to_vec(&raw).unwrap()).unwrap();
                assert!(stagemaster_project::Document::decode(&fs::read(&show).unwrap()).is_err());
            }
        }
        fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
        let run = dir.path().join("run");
        let error = rejection::rejected(
            Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"))
                .arg(&show)
                .arg("group")
                .arg(manifest)
                .arg(&run)
                .arg("--software-output"),
            &run,
        );
        if case == 5 {
            assert!(
                error.contains("/media/audioEditing/loopRegions/0/plays"),
                "{error}"
            );
        }
    }
}
#[test]
fn media_authority_generation_and_retries_do_not_duplicate_or_silently_restart_audio() {
    runtime().block_on(authority());
}
async fn authority() {
    let mut h = Harness::prepared(|path| Some(audio::write(path)));
    let session = h.session().await;
    let initial = h.state().await;
    let denied = control(&h, &session, 1, &initial, json!({"kind":"play"})).await;
    assert_eq!(denied["kind"], "rejected");
    assert_eq!(h.state().await["audio"]["frames"], "0");
    let acquired = h
        .command(
            &session,
            2,
            json!({"kind":"acquire","durationMs":60000,"takeover":false}),
        )
        .await;
    let accepted = control(&h, &session, 3, &acquired["state"], json!({"kind":"play"})).await;
    let playing = applied(&h, &accepted).await;
    assert_eq!(
        control(&h, &session, 3, &acquired["state"], json!({"kind":"play"})).await,
        accepted
    );
    for (serial, state, action) in [
        (4, acquired["state"].clone(), json!({"kind":"stop"})),
        (
            5,
            playing.clone(),
            json!({"kind":"seek","positionMs":5001,"playing":false}),
        ),
        (
            6,
            playing.clone(),
            json!({"kind":"seek","positionMs":u64::MAX,"playing":true}),
        ),
    ] {
        assert_eq!(
            control(&h, &session, serial, &state, action).await["kind"],
            "rejected"
        );
        assert_eq!(
            h.state().await["audio"]["instance"],
            playing["audio"]["instance"]
        );
    }
    let other = h.session().await;
    let takeover = h.acquire(&other, true).await;
    assert_eq!(
        control(&h, &session, 7, &playing, json!({"kind":"pause"})).await["kind"],
        "rejected"
    );
    let later = until(&h, |s| {
        s["state"]["media"][0]["positionMs"].as_u64().unwrap()
            > playing["media"][0]["positionMs"].as_u64().unwrap() + 100
    })
    .await;
    assert_eq!(later["state"]["media"][0]["status"], "Following");
    assert_eq!(later["state"]["owner"], takeover["state"]["owner"]);
    h.close().await; // Normal shutdown while the real native source is still being consumed.
}

#[test]
fn failed_seek_preparation_stops_its_media_group_without_stopping_independent_lighting() {
    runtime().block_on(failed_seek(3000));
}
#[test]
fn end_seek_needs_no_more_samples_but_replay_reports_a_damaged_private_resource() {
    runtime().block_on(failed_seek(5000));
}
async fn failed_seek(position_ms: u64) {
    let mut h = Harness::prepared(|path| Some(audio::write(path)));
    let session = h.session().await;
    let acquired = h.acquire(&session, false).await;
    let independent = start(&h, &session, 2, &acquired["state"]["revision"], 0).await;
    let accepted = control(
        &h,
        &session,
        3,
        &independent["state"],
        json!({"kind":"play"}),
    )
    .await;
    let state = applied(&h, &accepted).await;
    let copy = fs::read_dir(h.directory.path().join("run/store/media"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(copy, b"fault injection into owned temporary test copy").unwrap();
    let failure = control(
        &h,
        &session,
        4,
        &state,
        json!({"kind":"seek","positionMs":position_ms,"playing":true}),
    )
    .await;
    assert_eq!(failure["kind"], "accepted");
    if position_ms == 5000 {
        let ended = applied(&h, &failure).await;
        assert_eq!(ended["media"][0]["status"], "Stopped");
        assert_eq!(ended["audio"]["positionMs"], 5000);
        until(&h, |s| s["state"]["audio"]["status"] == "ended").await;
        let replay = control(&h, &session, 5, &ended, json!({"kind":"play"})).await;
        assert_eq!(replay["kind"], "accepted");
        let failed = until(&h, |s| {
            s["state"]["audio"]["status"] == "failed"
                && s["state"]["media"][0]["control"]["status"] == "failed"
        })
        .await;
        assert_eq!(failed["state"]["media"][0]["status"], "Stopped");
        assert_eq!(failed["state"]["sources"][0]["status"], "Running");
        assert_eq!(failed["state"]["fault"], false);
        until(&h, |s| s["frame"]["slots"][1] == 20).await;
        h.close().await;
        return;
    }
    let failed = until(&h, |s| {
        s["state"]["audio"]["status"] == "failed"
            && s["state"]["media"][0]["termination"]["applied"] == true
    })
    .await;
    assert_eq!(failed["state"]["media"][0]["control"]["status"], "failed");
    assert_eq!(
        failed["state"]["media"][0]["termination"]["reason"],
        "failed"
    );
    assert_eq!(failed["state"]["sources"][0]["status"], "Running");
    assert_eq!(failed["state"]["media"][0]["status"], "Stopped");
    assert_eq!(failed["state"]["fault"], false);
    h.close().await;
}
