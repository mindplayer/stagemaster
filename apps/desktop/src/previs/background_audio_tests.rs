use super::*;
use stagemaster_execution_client::{MediaAction, MediaCompletion};
use std::sync::atomic::AtomicBool;

fn add_music(document: &mut Document, root: &std::path::Path) {
    let wav = root.join("song.wav");
    let size = 8000_u32 * 2 * 3;
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
    fs::write(&wav, bytes).unwrap();
    let (digest, _) = stagemaster_audio::Resources::new(root.join("project.json.assets"))
        .import(&wav, "wav", None, &AtomicBool::new(false))
        .unwrap();
    let scene = document.view().scenes[0].id.clone();
    for command in [
        json!({"kind":"setAsset","asset":{"digest":digest,"fileName":"song.wav","extension":"wav","durationMs":3000}}),
        json!({"kind":"putMarker","marker":{"id":uuid::Uuid::new_v4().to_string(),"name":"开场","timeMs":0,"sceneId":scene,"fadeMs":0}}),
    ] {
        document
            .edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
}
#[tokio::test]
async fn audio_only_background_projects_use_the_same_readonly_stage_and_lighting_projection() {
    let shared = shared();
    let mut doc = shared.lock().unwrap().previs_document().unwrap();
    let temp =
        tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap();
    add_music(&mut doc, temp.path());
    let (_process, path) = launch(&doc, temp.path()).await;
    let observer = Background::connect(&path).await.unwrap();
    assert_eq!(observer.scene.fixtures.len(), 1);
    let mut controller = Client::open(&path).await.unwrap();
    assert!(
        controller
            .view()
            .observation
            .snapshot
            .unwrap()
            .state
            .owner
            .is_none()
    );
    controller.acquire(false).await.unwrap();
    let view = settled(&mut controller).await;
    let state = view.observation.snapshot.unwrap().state;
    controller
        .apply_media(
            &view.host_id,
            &state.revision,
            &state.media[0].id,
            &state.media[0].generation,
            MediaAction::Play {},
        )
        .await
        .unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let view = controller.refresh().await.unwrap();
        if view.observation.snapshot.unwrap().state.media[0]
            .control
            .as_ref()
            .is_some_and(|c| c.status == MediaCompletion::Applied)
        {
            break;
        }
        assert!(Instant::now() < end);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    fs::remove_file(temp.path().join("project.json")).unwrap();
    let lights = observer.lights().await.unwrap();
    assert_eq!(lights.len(), 1);
    assert!(lights[0].intensity > 0.0);
    drop(observer);
    let reopened = Background::connect(&path).await.unwrap();
    assert_eq!(reopened.host_id, view.host_id);
    assert_eq!(
        serde_json::to_value(reopened.lights().await.unwrap()).unwrap(),
        serde_json::to_value(lights).unwrap()
    );
    assert!(controller.refresh().await.unwrap().controlling);
    controller.shutdown().await.unwrap();
}
