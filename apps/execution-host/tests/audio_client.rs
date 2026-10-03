#![cfg(feature = "audio")]
mod support;
use stagemaster_execution_client::{
    Action, AudioOutput, Client, MediaAction, MediaStatus, Reader, View,
};
use support::*;

use support::client_audio::*;
#[test]
fn shared_client_and_readonly_reader_use_the_real_background_audio_group() {
    runtime().block_on(run());
}
async fn run() {
    let mut h = Harness::prepared(|p| Some(audio::write(p)));
    let path = h.directory.path().join("run/discovery.json");
    let mut client = Client::open(&path).await.unwrap();
    let mut reader = Reader::open(&path).await.unwrap();
    assert_eq!(
        reader.catalog().audio.as_ref().unwrap().output,
        AudioOutput::Software
    );
    reader.project().await.unwrap();
    let idle = reader.sample().await.unwrap().slots;
    assert!(h.state().await["owner"].is_null());
    assert!(!client.view().controlling);
    client.acquire(false).await.unwrap();
    let ready = settled(&mut client).await;
    let old_generation = ready.observation.snapshot.unwrap().state.media[0]
        .generation
        .clone();
    let playing = apply(&mut client, MediaAction::Play {}).await;
    let instance = playing
        .observation
        .snapshot
        .as_ref()
        .unwrap()
        .state
        .audio
        .as_ref()
        .unwrap()
        .instance
        .clone();
    client.release().await.unwrap();
    settled(&mut client).await;
    drop(client);
    let mut other = Client::open(&path).await.unwrap();
    assert!(other.view().session_id.is_none());
    let later = wait(&mut other, |v| {
        v.observation
            .snapshot
            .as_ref()
            .is_some_and(|s| s.state.media[0].position_ms > 100)
    })
    .await;
    let state = &later.observation.snapshot.as_ref().unwrap().state;
    assert_eq!(state.audio.as_ref().unwrap().instance, instance);
    assert!(state.owner.is_none());
    assert_eq!(state.media[0].status, MediaStatus::Following);
    other.acquire(false).await.unwrap();
    let stopped = verify_operations(&mut other, &mut reader, &old_generation, &idle).await;
    assert_eq!(
        h.state().await["owner"]["sessionId"].as_str(),
        stopped.session_id.as_deref()
    );
    h.close().await;
}

async fn verify_operations(
    other: &mut Client,
    reader: &mut Reader,
    old_generation: &str,
    idle: &[u8],
) -> View {
    let paused = apply(other, MediaAction::Pause {}).await;
    let state = &paused.observation.snapshot.as_ref().unwrap().state;
    assert_eq!(state.media[0].status, MediaStatus::Paused);
    other
        .apply_media(
            &paused.host_id,
            &state.revision,
            &state.media[0].id,
            old_generation,
            MediaAction::Stop {},
        )
        .await
        .unwrap();
    let refused = settled(other).await;
    assert_eq!(
        refused
            .record
            .as_ref()
            .unwrap()
            .outcome
            .as_ref()
            .unwrap()
            .kind,
        "rejected"
    );
    assert_eq!(
        refused.observation.snapshot.as_ref().unwrap().state.media[0].status,
        MediaStatus::Paused
    );
    let located = apply(
        other,
        MediaAction::Seek {
            position_ms: 3000,
            playing: false,
        },
    )
    .await;
    assert_eq!(
        located.observation.snapshot.as_ref().unwrap().state.media[0].position_ms,
        3000
    );
    assert_eq!(reader.sample().await.unwrap().slots[1], 40);
    let state = &located.observation.snapshot.as_ref().unwrap().state;
    let serial = located.record.as_ref().unwrap().serial.clone();
    assert!(
        other
            .apply(
                &located.host_id,
                &state.revision,
                &state.media[0].id,
                Action::Pause {}
            )
            .await
            .is_err()
    );
    assert!(
        other
            .apply_media(
                &located.host_id,
                &state.revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Seek {
                    position_ms: 5001,
                    playing: false
                }
            )
            .await
            .is_err()
    );
    assert_eq!(other.view().record.unwrap().serial, serial);
    let ended = apply(
        other,
        MediaAction::Seek {
            position_ms: 5000,
            playing: true,
        },
    )
    .await;
    let state = &ended.observation.snapshot.as_ref().unwrap().state;
    assert_eq!(state.media[0].status, MediaStatus::Stopped);
    let native = state.audio.as_ref().unwrap();
    assert_eq!(
        native.status,
        stagemaster_execution_client::AudioStatus::Ended
    );
    assert_eq!(native.position_ms, 5000);
    assert_eq!(reader.sample().await.unwrap().slots.as_slice(), idle);
    let stopped = apply(other, MediaAction::Stop {}).await;
    assert_eq!(
        stopped.observation.snapshot.as_ref().unwrap().state.media[0].status,
        MediaStatus::Stopped
    );
    stopped
}
