#![cfg(feature = "audio")]
mod support;
use stagemaster_execution_client::{AudioStatus, Client, MediaAction, Reader};
use support::{client_audio::*, *};

#[test]
fn shared_recovery_uses_existing_control_and_reader_stays_attached_to_the_same_host() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let path = h.directory.path().join("run/discovery.json");
        let mut client = Client::open(&path).await.unwrap();
        let mut reader = Reader::open(&path).await.unwrap();
        let host = client.view().host_id;
        assert!(reader.catalog().audio.as_ref().unwrap().provider_recovery);
        client.acquire(false).await.unwrap();
        let initial = apply(
            &mut client,
            MediaAction::Seek {
                position_ms: 1000,
                playing: false,
            },
        )
        .await;
        let original = initial
            .observation
            .snapshot
            .unwrap()
            .state
            .audio
            .unwrap()
            .instance;
        let rebound = apply(&mut client, MediaAction::Recover { position_ms: 0 }).await;
        assert_eq!(rebound.host_id, host);
        let paused = wait(&mut client, |v| {
            v.observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .audio
                .as_ref()
                .unwrap()
                .status
                == AudioStatus::Paused
        })
        .await;
        let state = paused.observation.snapshot.unwrap().state;
        let native = state.audio.unwrap();
        assert_eq!(native.position_ms, 0);
        assert_ne!(native.instance, original);
        assert!(native.problem.is_none());
        reader.sample().await.unwrap();
        client.release().await.unwrap();
        settled(&mut client).await;
        drop(client);
        let mut client = Client::open(&path).await.unwrap();
        assert!(!client.view().controlling);
        let before = client.view();
        let state = &before.observation.snapshot.as_ref().unwrap().state;
        assert!(
            client
                .apply_media(
                    &host,
                    &state.revision,
                    &state.media[0].id,
                    &state.media[0].generation,
                    MediaAction::Recover { position_ms: 0 }
                )
                .await
                .is_err()
        );
        client.acquire(false).await.unwrap();
        apply(&mut client, MediaAction::Play {}).await;
        let playing = wait(&mut client, |v| {
            v.observation
                .snapshot
                .as_ref()
                .unwrap()
                .state
                .audio
                .as_ref()
                .unwrap()
                .position_ms
                > 50
        })
        .await;
        assert_eq!(playing.host_id, host);
        apply(&mut client, MediaAction::Stop {}).await;
        h.close().await;
    });
}
