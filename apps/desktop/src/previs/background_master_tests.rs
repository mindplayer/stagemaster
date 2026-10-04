use super::*;
use stagemaster_execution_client::OutputAction;

#[tokio::test]
async fn background_master_is_projected_once_and_ignores_editor_preview_blackout() {
    let shared = shared();
    let doc = {
        let mut session = shared.lock().unwrap();
        let view = session.previs_document().unwrap().view();
        let generation = session.previs_revision().generation;
        session
            .edit(
                generation,
                serde_json::from_value(json!({"op":"setSceneValue","sceneId":view.scenes[0].id,
            "fixtureId":view.fixtures[0].id,"attribute":"dimmer","mode":"literal","value":65535}))
                .unwrap(),
            )
            .unwrap();
        let snapshot = session
            .output_request(crate::output_control::Request::Snapshot)
            .unwrap();
        session
            .output_request(crate::output_control::Request::Set {
                epoch: snapshot.epoch,
                serial: snapshot.serial + 1,
                percent: 0,
                blackout: true,
            })
            .unwrap();
        session.previs_document().unwrap()
    };
    let temp =
        tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap();
    let (_process, discovery) = launch(&doc, temp.path()).await;
    let binding = SharedBackground::default();
    let observer = Background::connect(&discovery).await.unwrap();
    let host = observer.host_id.clone();
    binding.lock().unwrap().install(observer).unwrap();
    let mut controller = Client::open(&discovery).await.unwrap();
    controller.acquire(false).await.unwrap();
    let v = settled(&mut controller).await;
    controller
        .apply(
            &host,
            &v.observation.snapshot.unwrap().state.revision,
            &v.catalog.sources[0].id,
            Action::Start {
                step: v.catalog.sources[0].steps[0].id.clone(),
            },
        )
        .await
        .unwrap();
    let v = settled(&mut controller).await;
    controller
        .output(
            &host,
            &v.observation.snapshot.unwrap().state.revision,
            OutputAction::Level { percent: 50 },
        )
        .await
        .unwrap();
    settled(&mut controller).await;
    {
        let mut session = shared.lock().unwrap();
        let generation = session.previs_revision().generation;
        session
            .set_previs_source(
                generation,
                Source::Background {
                    host_id: host.clone(),
                },
            )
            .unwrap();
    }
    let server = Server::with_background(shared.clone(), Arc::new(|| {}), binding)
        .await
        .unwrap();
    let http = client();
    json_get(&http, &server, "/v1/scene").await;
    let half = json_get(&http, &server, "/v1/frame").await;
    assert_eq!(half["status"], "background");
    let intensity = half["lights"][0]["intensity"].as_f64().unwrap();
    assert!((intensity - 128.0 / 255.0).abs() < 0.00001, "{half}");
    let v = settled(&mut controller).await;
    controller
        .output(
            &host,
            &v.observation.snapshot.unwrap().state.revision,
            OutputAction::Blackout { enabled: true },
        )
        .await
        .unwrap();
    settled(&mut controller).await;
    let dark = json_get(&http, &server, "/v1/frame").await;
    assert_eq!(dark["lights"][0]["intensity"], 0.0);
    assert_eq!(dark["lights"][0]["color"], half["lights"][0]["color"]);
    controller.shutdown().await.unwrap();
}
