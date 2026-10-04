#![cfg(feature = "audio")]
mod support;
use serde_json::json;
use stagemaster_execution_client::{Client, MediaAction, OutputAction, View};
use support::{client_audio::*, *};

fn audio(v: &View) -> &stagemaster_execution_client::AudioState {
    v.observation
        .snapshot
        .as_ref()
        .unwrap()
        .state
        .audio
        .as_ref()
        .unwrap()
}
#[test]
fn output_blackout_leaves_the_original_music_instance_and_consumption_clock_running() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(loops::write(p, &json!({"kind":"untilExit"}))));
        let mut client = Client::open(&h.directory.path().join("run/discovery.json"))
            .await
            .unwrap();
        client.acquire(false).await.unwrap();
        let playing = apply(&mut client, MediaAction::Play {}).await;
        let instance = audio(&playing).instance.clone();
        let frames = audio(&playing).frames.parse::<u64>().unwrap();
        let before = &playing.observation.snapshot.as_ref().unwrap().state;
        let generation = before.media[0].generation.clone();
        client
            .output(
                &playing.host_id,
                &before.revision,
                OutputAction::Blackout { enabled: true },
            )
            .await
            .unwrap();
        let dark = settled(&mut client).await;
        assert!(
            dark.observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .output
                .as_ref()
                .unwrap()
                .blackout
        );
        let advanced = wait(&mut client, |v| {
            audio(v).frames.parse::<u64>().unwrap() > frames + 1000
        })
        .await;
        assert_eq!(audio(&advanced).instance, instance);
        assert_eq!(
            audio(&advanced).status,
            stagemaster_execution_client::AudioStatus::Playing
        );
        assert_eq!(
            advanced.observation.snapshot.as_ref().unwrap().state.media[0].generation,
            generation
        );
        apply(&mut client, MediaAction::Stop {}).await;
        h.close().await;
    });
}
