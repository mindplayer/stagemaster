use super::tests::{Cleanup, binary, connected, document, runtime};
use super::*;
use crate::execution::media::AudioInput;
use stagemaster_execution_client::{AudioOutput, MediaAction, MediaCompletion};
use std::{fs, path::Path, sync::atomic::AtomicBool, time::Instant};

pub(super) fn music(root: &Path) -> (Document, AudioInput) {
    let source = root.join("music.wav");
    let size = 8000_u32 * 2 * 5;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + size).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(8000_u32.to_le_bytes());
    bytes.extend(16000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    bytes.resize(44 + usize::try_from(size).unwrap(), 0);
    fs::write(&source, bytes).unwrap();
    let (digest, _) = stagemaster_audio::Resources::new(root.join("cache"))
        .import(&source, "wav", None, &AtomicBool::new(false))
        .unwrap();
    let mut doc = document();
    let scene = doc.view().scenes[0].id.clone();
    for command in [
        serde_json::json!({"kind":"setAsset","asset":{"digest":digest,"fileName":"music.wav","extension":"wav","durationMs":5000}}),
        serde_json::json!({"kind":"putMarker","marker":{"id":uuid::Uuid::new_v4().to_string(),"name":"入场","timeMs":0,"sceneId":scene,"fadeMs":0}}),
    ] {
        doc.edit(
            serde_json::from_value(serde_json::json!({"op":"audio","command":command})).unwrap(),
        )
        .unwrap();
    }
    (
        doc,
        AudioInput {
            source,
            output: AudioOutput::Software,
        },
    )
}
pub(super) async fn operation(manager: &mut Manager, action: MediaAction) -> View {
    let view = connected(manager).await.runtime.unwrap();
    let state = view.observation.snapshot.unwrap().state;
    manager
        .apply_media(
            &view.host_id,
            &state.revision,
            &state.media[0].id,
            &state.media[0].generation,
            action,
        )
        .await
        .unwrap();
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let view = connected(manager).await.runtime.unwrap();
        let requested = view
            .record
            .as_ref()
            .unwrap()
            .outcome
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .media[0]
            .control
            .as_ref()
            .unwrap();
        let actual = view.observation.snapshot.as_ref().unwrap().state.media[0]
            .control
            .as_ref()
            .unwrap();
        if actual.request == requested.request && actual.status == MediaCompletion::Applied {
            return view;
        }
        assert!(Instant::now() < end, "{view:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
#[test]
fn desktop_music_has_portable_resources_and_keeps_editor_exclusion_through_reconnect() {
    runtime().block_on(async {
        let temp =
            tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp"))
                .unwrap();
        let root = temp.path().join("execution");
        let (doc, input) = music(temp.path());
        let original = input.source.clone();
        let mut manager = Manager::new(root.clone(), binary());
        let reservation = manager.reserve_editor_audio().unwrap();
        let mut rival = Manager::new(root.clone(), binary());
        assert!(
            rival
                .prepare(doc.clone(), vec![Selection::AudioTimeline {}], None)
                .await
                .is_err()
        );
        drop(reservation);
        manager
            .prepare(doc, vec![Selection::AudioTimeline {}], Some(input))
            .await
            .unwrap();
        let _cleanup = Cleanup(manager.child.take().unwrap());
        let view = connected(&mut manager).await.runtime.unwrap();
        assert_eq!(
            view.catalog.audio.as_ref().unwrap().output,
            AudioOutput::Software
        );
        let run = manager.run.clone().unwrap();
        assert!(run.join("project.json.assets").is_dir());
        assert!(manager.reserve_editor_audio().is_err());
        manager.acquire(false).await.unwrap();
        let playing = operation(&mut manager, MediaAction::Play {}).await;
        drop(manager);
        fs::remove_file(original).unwrap();
        fs::remove_dir_all(temp.path().join("cache")).unwrap();
        fs::remove_dir_all(run.join("project.json.assets")).unwrap();
        fs::remove_file(run.join("project.json")).unwrap();
        let mut reopened = Manager::new(root.clone(), binary());
        let read = connected(&mut reopened).await.runtime.unwrap();
        assert_eq!(read.host_id, playing.host_id);
        assert!(!read.controlling);
        assert!(reopened.reserve_editor_audio().is_err());
        reopened.acquire(true).await.unwrap();
        let paused = operation(&mut reopened, MediaAction::Pause {}).await;
        assert_eq!(
            paused
                .observation
                .snapshot
                .unwrap()
                .state
                .audio
                .unwrap()
                .status,
            stagemaster_execution_client::AudioStatus::Paused
        );
        operation(
            &mut reopened,
            MediaAction::Seek {
                position_ms: 1500,
                playing: false,
            },
        )
        .await;
        reopened.shutdown(&playing.host_id).await.unwrap();
        assert_eq!(reopened.poll().await.phase, "empty");
        assert!(reopened.reserve_editor_audio().is_ok());
    });
}
#[test]
fn missing_configuration_and_damaged_music_do_not_publish_background_records() {
    runtime().block_on(async {
        let temp =
            tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp"))
                .unwrap();
        let root = temp.path().join("execution");
        let mut manager = Manager::new(root.clone(), binary());
        let (doc, input) = music(temp.path());
        assert!(
            manager
                .prepare(doc.clone(), vec![Selection::AudioTimeline {}], None)
                .await
                .is_err()
        );
        assert!(!root.join("current").exists());
        fs::write(&input.source, b"broken music").unwrap();
        assert!(
            manager
                .prepare(doc, vec![Selection::AudioTimeline {}], Some(input))
                .await
                .is_err()
        );
        assert!(!root.join("current").exists());
        let reservation = manager.reserve_editor_audio();
        assert!(
            reservation.is_ok(),
            "{reservation:?}; root={}",
            root.display()
        );
    });
}
