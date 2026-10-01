use super::*;
use crate::{preview, session::Session};
use serde_json::{Value, json};
use stagemaster_project::Document;
use std::sync::atomic::{AtomicUsize, Ordering};

fn shared() -> SharedSession {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    let mut doc = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let fixture = doc.view().fixtures[0].id.clone();
    doc.edit(serde_json::from_value(json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":fixture,"spaceId":null,"positionMeters":{"x":"1","y":"2","z":"3"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}})).unwrap()).unwrap();
    Arc::new(Mutex::new(Session::from_previs_test_document(doc)))
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}
fn get(client: &reqwest::Client, server: &Server, path: &str) -> reqwest::RequestBuilder {
    client
        .get(format!("http://{}{path}", server.address))
        .header(header::AUTHORIZATION, &server.context.authorization)
}
async fn json_get(client: &reqwest::Client, server: &Server, path: &str) -> Value {
    get(client, server, path)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}
async fn start(shared: SharedSession) -> Server {
    Server::start(shared, Arc::new(|| {})).await.unwrap()
}

#[tokio::test]
async fn loopback_authentication_origin_and_closed_capabilities_are_enforced() {
    let shared = shared();
    let bridge = start(shared.clone()).await;
    let client = client();
    assert!(bridge.address.ip().is_loopback());
    assert!(!bridge.status(Source::Defaults, Instant::now()).connected);
    assert_eq!(
        client
            .get(format!("http://{}/v1/scene", bridge.address))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&client, &bridge, "/v1/scene")
            .header(header::ORIGIN, "http://127.0.0.1")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&client, &bridge, "/v1/frame")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let scene = json_get(&client, &bridge, "/v1/scene").await;
    assert_eq!(scene["protocol"], 2);
    assert_eq!(scene["scene"]["fixtures"].as_array().unwrap().len(), 1);
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["version"], scene["version"]);
    assert_eq!(frame["status"], "editing");
    assert!(!frame["canEdit"].as_bool().unwrap());
    assert!(bridge.status(Source::Defaults, Instant::now()).connected);
    assert!(
        !bridge
            .status(Source::Defaults, Instant::now() + Duration::from_secs(3))
            .connected
    );
    assert_eq!(
        get(&client, &bridge, "/v1/execute")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    let old_authorization = bridge.context.authorization.clone();
    let old_context = bridge.context.clone();
    drop(bridge);
    assert!(!*old_context.active.lock().unwrap());
    let next = start(shared).await;
    assert_ne!(next.context.authorization, old_authorization);
    assert_eq!(
        client
            .get(format!("http://{}/v1/scene", next.address))
            .header(header::AUTHORIZATION, old_authorization)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn placement_is_one_guarded_history_edit_and_cannot_create_or_reparent() {
    let shared = shared();
    let changed = Arc::new(AtomicUsize::new(0));
    let notifications = changed.clone();
    let bridge = Server::start(
        shared.clone(),
        Arc::new(move || {
            notifications.fetch_add(1, Ordering::Relaxed);
        }),
    )
    .await
    .unwrap();
    let client = client();
    let scene = json_get(&client, &bridge, "/v1/scene").await;
    let mut body = json!({"bridgeId":scene["bridgeId"],"version":scene["version"],"generation":scene["generation"],"placement":scene["scene"]["fixtures"][0]["placement"]});
    body["placement"]["positionMeters"]["x"] = json!("4.2");
    let post = |body: &Value| {
        client
            .post(format!("http://{}/v1/placement", bridge.address))
            .header(header::AUTHORIZATION, &bridge.context.authorization)
            .json(body)
    };
    assert_eq!(
        post(&body).send().await.unwrap().status(),
        StatusCode::CONFLICT
    );
    {
        let mut session = shared.lock().unwrap();
        let revision = session.previs_revision();
        session
            .set_previs_editing(revision.generation, true)
            .unwrap();
    }
    for (key, value) in [
        ("fixtureId", json!("missing")),
        ("spaceId", json!("missing")),
    ] {
        let mut invalid = body.clone();
        invalid["placement"][key] = value;
        assert_eq!(
            post(&invalid).send().await.unwrap().status(),
            StatusCode::CONFLICT
        );
    }
    let mut invalid = body.clone();
    invalid["unexpected"] = json!(true);
    assert_eq!(
        post(&invalid).send().await.unwrap().status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut invalid = body.clone();
    invalid["placement"]["positionMeters"]["z"] = json!("NaN");
    assert_eq!(
        post(&invalid).send().await.unwrap().status(),
        StatusCode::CONFLICT
    );
    assert_eq!(post(&body).send().await.unwrap().status(), StatusCode::OK);
    assert_eq!(changed.load(Ordering::Relaxed), 1);
    // Retrying an already-applied edit is rejected instead of creating another undo step.
    assert_eq!(
        post(&body).send().await.unwrap().status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        get(&client, &bridge, "/v1/frame")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let next = json_get(&client, &bridge, "/v1/scene").await;
    assert_eq!(
        next["scene"]["fixtures"][0]["placement"]["positionMeters"]["x"],
        "4.2"
    );
    let mut session = shared.lock().unwrap();
    let generation = session.previs_revision().generation;
    session.history(generation, false).unwrap();
    assert_eq!(
        session.previs_document().unwrap().view().stage.placements[0]
            .position_meters
            .x,
        "1"
    );
    assert_eq!(
        serde_json::to_value(session.snapshot()).unwrap()["canUndo"],
        false
    );
    let generation = session.previs_revision().generation;
    session.history(generation, true).unwrap();
    assert_eq!(
        session.previs_document().unwrap().view().stage.placements[0]
            .position_meters
            .x,
        "4.2"
    );
}

fn paused_playback(shared: &SharedSession) -> Vec<stagemaster_previs::Light> {
    let doc = shared.lock().unwrap().previs_document().unwrap();
    let sequence_id = doc.view().sequences[0].id.clone();
    {
        let mut session = shared.lock().unwrap();
        let generation = session.previs_revision().generation;
        session
            .set_previs_source(generation, Source::Playback)
            .unwrap();
        let loaded = session
            .preview(preview::Request::Load {
                generation,
                sequence_id,
            })
            .unwrap();
        let epoch = u32::try_from(
            serde_json::to_value(loaded).unwrap()["epoch"]
                .as_u64()
                .unwrap(),
        )
        .unwrap();
        session
            .preview(preview::Request::Control {
                epoch,
                serial: 1,
                command: preview::Command::Next,
            })
            .unwrap();
        session
            .preview(preview::Request::Control {
                epoch,
                serial: 2,
                command: preview::Command::Pause,
            })
            .unwrap();
        let snapshot = session.previs_frame().unwrap();
        stagemaster_previs::playback_lights(
            &doc,
            snapshot.playback.unwrap().output.as_ref().unwrap(),
        )
    }
}

#[tokio::test]
async fn frame_uses_same_paused_player_values_and_rejects_stale_geometry_and_plan() {
    let shared = shared();
    let expected = paused_playback(&shared);
    let bridge = start(shared.clone()).await;
    let client = client();
    json_get(&client, &bridge, "/v1/scene").await;
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["status"], "paused");
    assert_eq!(frame["lights"], serde_json::to_value(expected).unwrap());
    {
        let mut session = shared.lock().unwrap();
        let generation = session.previs_revision().generation;
        session
            .edit(
                generation,
                stagemaster_project::EditCommand::SetInfo {
                    name: "新名".into(),
                    description: String::new(),
                },
            )
            .unwrap();
    }
    assert_eq!(
        get(&client, &bridge, "/v1/frame")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    json_get(&client, &bridge, "/v1/scene").await;
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["status"], "stalePlayback");
    assert_eq!(frame["lights"], json!([]));
    {
        let mut session = shared.lock().unwrap();
        let revision = session.previs_revision();
        session
            .set_previs_editing(revision.generation, true)
            .unwrap();
        let placement = session.previs_document().unwrap().view().stage.placements[0].clone();
        assert!(
            session
                .previs_place(PlacementRequest {
                    bridge_id: bridge.context.bridge_id.clone(),
                    generation: revision.generation,
                    version: revision.content.to_string(),
                    placement,
                })
                .is_err()
        );
    }
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["canEdit"], false);
    drop(bridge);
    let snapshot = serde_json::to_value(
        shared
            .lock()
            .unwrap()
            .preview(preview::Request::Snapshot)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(snapshot["loaded"]["status"], "paused");
    assert_eq!(snapshot["loaded"]["stale"], true);
}

#[tokio::test]
async fn busy_session_and_projection_saturation_fail_promptly_without_queued_work() {
    let shared = shared();
    let bridge = start(shared.clone()).await;
    let client = client();
    let (ready, ready_receive) = oneshot::channel();
    let (release, released) = std::sync::mpsc::channel();
    let locked_session = shared.clone();
    let holder = std::thread::spawn(move || {
        let _held = locked_session.lock().unwrap();
        ready.send(()).unwrap();
        released.recv().unwrap();
    });
    ready_receive.await.unwrap();
    let began = Instant::now();
    assert_eq!(
        get(&client, &bridge, "/v1/scene")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(began.elapsed() < Duration::from_secs(2));
    release.send(()).unwrap();
    holder.join().unwrap();
    let permits = bridge.context.workers.acquire_many(2).await.unwrap();
    assert_eq!(
        get(&client, &bridge, "/v1/scene")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(permits);
    assert_eq!(
        get(&client, &bridge, "/v1/scene")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn cancelling_http_wait_does_not_release_capacity_while_projection_is_still_running() {
    let bridge = start(shared()).await;
    let context = bridge.context.clone();
    let (ready, ready_receive) = oneshot::channel();
    let (release, released) = std::sync::mpsc::channel();
    let waiter = tokio::spawn(blocking(context.clone(), move |_| {
        ready.send(()).unwrap();
        released.recv_timeout(Duration::from_secs(5)).unwrap();
        Ok(())
    }));
    ready_receive.await.unwrap();
    waiter.abort();
    assert!(waiter.await.unwrap_err().is_cancelled());
    assert_eq!(context.workers.available_permits(), 1);
    assert!(context.workers.try_acquire_many(2).is_err());
    release.send(()).unwrap();
    let permits = tokio::time::timeout(Duration::from_secs(2), context.workers.acquire_many(2))
        .await
        .unwrap()
        .unwrap();
    drop(permits);
}

#[tokio::test]
async fn explicit_scene_source_resolves_presets_and_deleted_scene_is_visible_as_missing() {
    let shared = shared();
    let (id, expected) = {
        let mut session = shared.lock().unwrap();
        let doc = session.previs_document().unwrap();
        let id = doc.view().scenes[0].id.clone();
        let generation = session.previs_revision().generation;
        session
            .set_previs_source(
                generation,
                Source::Scene {
                    scene_id: id.clone(),
                },
            )
            .unwrap();
        let expected = stagemaster_previs::editing_lights(&doc, Some(&id)).unwrap();
        (id, expected)
    };
    let bridge = start(shared.clone()).await;
    let client = client();
    json_get(&client, &bridge, "/v1/scene").await;
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["source"], json!({"kind":"scene","sceneId":id}));
    assert_eq!(frame["lights"], serde_json::to_value(expected).unwrap());
    {
        let mut session = shared.lock().unwrap();
        let doc = session.previs_document().unwrap();
        let mut commands = doc
            .view()
            .sequences
            .into_iter()
            .map(|sequence| stagemaster_project::EditCommand::Sequence {
                command: stagemaster_project::SequenceEdit::Remove { id: sequence.id },
            })
            .collect::<Vec<_>>();
        commands.push(stagemaster_project::EditCommand::RemoveScene { id: id.clone() });
        let generation = session.previs_revision().generation;
        session
            .edit(
                generation,
                stagemaster_project::EditCommand::Batch { commands },
            )
            .unwrap();
        // Re-selecting a deleted scene is rejected; the old source stays visibly invalid.
        let generation = session.previs_revision().generation;
        assert!(
            session
                .set_previs_source(
                    generation,
                    Source::Scene {
                        scene_id: id.clone()
                    }
                )
                .is_err()
        );
        assert_eq!(session.previs_source(), Source::Scene { scene_id: id });
    }
    json_get(&client, &bridge, "/v1/scene").await;
    let frame = json_get(&client, &bridge, "/v1/frame").await;
    assert_eq!(frame["status"], "missingScene");
    assert_eq!(frame["lights"], json!([]));
    assert!(
        bridge
            .status(Source::Defaults, Instant::now())
            .problem
            .is_some()
    );
}

#[path = "output_master_tests.rs"]
mod output_master_tests;
