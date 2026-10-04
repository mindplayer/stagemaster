use super::*;
use crate::previs::background::{Background, SharedBackground};
use stagemaster_execution_client::{Action, Client, View};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn settled(client: &mut Client) -> View {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let view = observed(client, end).await;
        if !view.pending {
            return view;
        }
        assert!(Instant::now() < end);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn transient_observation(error: &str, before_deadline: bool) -> bool {
    before_deadline && error == "后台请求未成功（503），请核对连接与原回执"
}
async fn observed(client: &mut Client, end: Instant) -> View {
    loop {
        match client.refresh().await {
            Ok(view) => return view,
            Err(error) if transient_observation(&error, Instant::now() < end) => {
                // Only repeat this read-only observation, not any control command.
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => panic!("{error}"),
        }
    }
}
#[test]
fn observation_wait_is_bounded_and_only_accepts_explicit_busy() {
    let busy = "后台请求未成功（503），请核对连接与原回执";
    assert!(transient_observation(busy, true));
    assert!(!transient_observation(busy, false));
    for error in [
        "后台请求未成功（409），请核对连接与原回执",
        "后台连接未响应；已发送操作须核对原回执",
        "后台响应格式无效",
    ] {
        assert!(!transient_observation(error, true));
    }
}
async fn launch(document: &Document, root: &std::path::Path) -> (Process, PathBuf) {
    let project = root.join("project.json");
    let sources = root.join("sources.json");
    fs::write(&project, document.encode().unwrap()).unwrap();
    let music = document.audio_timeline().is_some();
    let selection = if music {
        json!({"kind":"audioTimeline"})
    } else {
        json!({"kind":"scene","id":document.view().scenes[0].id})
    };
    let mut manifest = json!({"version":if music {2} else {1},"sources":[{
        "id":uuid::Uuid::new_v4().to_string(),"priority":0,"selection":selection
    }]});
    if music {
        manifest["audio"] = json!({"output":"software"});
    }
    fs::write(&sources, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("stagemaster-execution-host");
    let process = Process(
        Command::new(binary)
            .arg(&project)
            .arg("group")
            .arg(sources)
            .arg(root.join("run"))
            .arg("--software-output")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let discovery = root.join("run/discovery.json");
    let end = Instant::now() + Duration::from_secs(8);
    while !discovery.exists() {
        assert!(Instant::now() < end);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    (process, discovery)
}

#[tokio::test]
async fn real_background_scene_is_frozen_readonly_and_survives_renderer_bridge_restart() {
    let shared = shared();
    let doc = shared.lock().unwrap().previs_document().unwrap();
    let temp =
        tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap();
    let (_process, discovery) = launch(&doc, temp.path()).await;
    let binding = SharedBackground::default();
    let observer = Background::connect(&discovery).await.unwrap();
    let host_id = observer.host_id.clone();
    binding.lock().unwrap().install(observer).unwrap();
    let mut controller = Client::open(&discovery).await.unwrap();
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
    let acquired = settled(&mut controller).await;
    controller
        .apply(
            &host_id,
            &acquired.observation.snapshot.unwrap().state.revision,
            &acquired.catalog.sources[0].id,
            Action::Start {
                step: acquired.catalog.sources[0].steps[0].id.clone(),
            },
        )
        .await
        .unwrap();
    let started = settled(&mut controller).await;
    let source = Source::Background {
        host_id: host_id.clone(),
    };
    {
        let mut s = shared.lock().unwrap();
        let generation = s.previs_revision().generation;
        s.set_previs_source(generation, source.clone()).unwrap();
        let mut placement = doc.view().stage.placements[0].clone();
        placement.position_meters.x = "99".into();
        let version = s.previs_revision().content.to_string();
        assert!(
            s.place_from_viewport(generation, &version, placement.clone())
                .is_err()
        );
        s.edit(
            generation,
            serde_json::from_value(
                json!({"op":"stage","command":{"op":"putPlacement","placement":placement}}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    fs::remove_file(temp.path().join("project.json")).unwrap();
    let bridge = Server::with_background(shared.clone(), Arc::new(|| {}), binding.clone())
        .await
        .unwrap();
    let http = client();
    let scene = json_get(&http, &bridge, "/v1/scene").await;
    assert_eq!(scene["generation"], 0);
    assert_eq!(scene["scene"]["fixtures"][0]["originMeters"][0], 1.0);
    let frame = json_get(&http, &bridge, "/v1/frame").await;
    assert_eq!(frame["status"], "background");
    assert_eq!(frame["canEdit"], false);
    assert_eq!(frame["lights"].as_array().unwrap().len(), 1);
    let raw = controller.refresh().await.unwrap();
    drop(bridge);
    tokio::time::sleep(Duration::from_millis(70)).await;
    let reopened = Server::with_background(shared.clone(), Arc::new(|| {}), binding)
        .await
        .unwrap();
    let new_scene = json_get(&http, &reopened, "/v1/scene").await;
    assert_ne!(new_scene["bridgeId"], scene["bridgeId"]);
    let again = json_get(&http, &reopened, "/v1/frame").await;
    assert_eq!(again["lights"], frame["lights"]);
    let after = controller.refresh().await.unwrap();
    assert_eq!(after.host_id, host_id);
    assert_eq!(
        after.observation.snapshot.as_ref().unwrap().state.revision,
        started.observation.snapshot.unwrap().state.revision
    );
    assert_ne!(
        after.observation.snapshot.as_ref().unwrap().cycles,
        raw.observation.snapshot.unwrap().cycles
    );
    assert!(after.controlling);
    {
        let mut s = shared.lock().unwrap();
        let generation = s.previs_revision().generation;
        s.set_previs_source(generation, Source::Defaults).unwrap();
    }
    let editing = json_get(&http, &reopened, "/v1/scene").await;
    assert_ne!(editing["generation"], 0);
    assert_eq!(editing["scene"]["fixtures"][0]["originMeters"][0], 99.0);
    controller.shutdown().await.unwrap();
}

#[path = "background_audio_tests.rs"]
mod audio;

#[path = "background_master_tests.rs"]
mod master;
