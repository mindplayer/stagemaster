mod support;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use stagemaster_execution_client::{Action, Client, MediaAction, MediaCompletion};
use std::{
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use support::*;
struct Proxy {
    base: String,
    http: reqwest::Client,
    corrupt: AtomicBool,
    submissions: AtomicUsize,
    old_observation: Mutex<Option<serde_json::Value>>,
}
async fn forward(State(proxy): State<Arc<Proxy>>, request: Request<Body>) -> Response {
    let path = request.uri().path();
    let command = path.ends_with("/commands");
    let observation = path.ends_with("/state");
    if command {
        proxy.submissions.fetch_add(1, Ordering::SeqCst);
    }
    let destination = format!("{}{}", proxy.base, path);
    let builder = proxy
        .http
        .request(request.method().clone(), destination)
        .headers(request.headers().clone());
    let body = to_bytes(request.into_body(), 8192).await.unwrap();
    let response = builder.body(body).send().await.unwrap();
    let status = response.status();
    let bytes = response.bytes().await.unwrap();
    if command && proxy.corrupt.swap(false, Ordering::SeqCst) {
        // The real host admitted the command; the client cannot decode its HTTP reply.
        return (StatusCode::OK, "{truncated").into_response();
    }
    if observation && let Some(old) = proxy.old_observation.lock().unwrap().clone() {
        return axum::Json(old).into_response();
    }
    (status, bytes).into_response()
}
#[test]
fn admitted_command_with_lost_reply_recovers_only_through_original_receipt() {
    runtime().block_on(run(false));
}
#[test]
fn accepted_audio_with_lost_reply_is_not_replayed_or_confused_with_actual_completion() {
    runtime().block_on(run(true));
}
async fn run(audio: bool) {
    let mut h = if audio {
        Harness::prepared(|p| Some(support::audio::write(p)))
    } else {
        Harness::start_group()
    };
    let discovery_path = h.directory.path().join("run/discovery.json");
    let (proxy, task) = install_proxy(&h, &discovery_path).await;
    let mut control = Client::open(&discovery_path).await.unwrap();
    control.acquire(false).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    while control.refresh().await.unwrap().pending {
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let before = control.view();
    let source = &before.catalog.sources[0];
    let revision = &before.observation.snapshot.as_ref().unwrap().state.revision;
    let older = ok(h.get("/state")).await;
    *proxy.old_observation.lock().unwrap() = Some(older);
    proxy.corrupt.store(true, Ordering::SeqCst);
    let result = if audio {
        let state = &before.observation.snapshot.as_ref().unwrap().state;
        control
            .apply_media(
                &before.host_id,
                revision,
                &state.media[0].id,
                &state.media[0].generation,
                MediaAction::Play {},
            )
            .await
    } else {
        control
            .apply(
                &before.host_id,
                revision,
                &source.id,
                Action::Start {
                    step: source.steps[0].id.clone(),
                },
            )
            .await
    };
    assert!(result.is_err());
    assert!(control.view().pending);
    assert!(control.release().await.is_err());
    let pending = control.view().record.unwrap().serial;
    loop {
        let view = control.refresh().await.unwrap();
        if !view.pending {
            assert_eq!(view.record.as_ref().unwrap().serial, pending);
            if audio {
                let state = view.observation.snapshot.unwrap().state;
                assert_eq!(
                    state.media[0].control.as_ref().unwrap().status,
                    MediaCompletion::Pending
                );
                assert_eq!(
                    view.record.as_ref().unwrap().outcome.as_ref().unwrap().kind,
                    "accepted"
                );
            } else {
                assert_eq!(
                    view.observation.snapshot.unwrap().state.sources[0]
                        .status
                        .as_deref(),
                    Some("Running")
                );
            }
            break;
        }
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2); // acquire + start, no re-send.
    if audio {
        *proxy.old_observation.lock().unwrap() = None;
        loop {
            let view = control.refresh().await.unwrap();
            if view.observation.snapshot.unwrap().state.media[0]
                .control
                .as_ref()
                .is_some_and(|c| c.status == MediaCompletion::Applied)
            {
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(proxy.submissions.load(Ordering::SeqCst), 2);
    }
    task.abort();
    h.close().await;
}

async fn install_proxy(
    h: &Harness,
    path: &std::path::Path,
) -> (Arc<Proxy>, tokio::task::JoinHandle<()>) {
    let actual = h.discovery["url"].as_str().unwrap();
    let base = actual.split("/v2/").next().unwrap().to_owned();
    let proxy = Arc::new(Proxy {
        base,
        http: client(),
        corrupt: AtomicBool::new(false),
        submissions: AtomicUsize::new(0),
        old_observation: Mutex::new(None),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = Router::new().fallback(forward).with_state(proxy.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let mut discovery = h.discovery.clone();
    discovery["url"] = format!(
        "http://{address}/v2/{}",
        discovery["hostId"].as_str().unwrap()
    )
    .into();
    fs::write(path, serde_json::to_vec(&discovery).unwrap()).unwrap();
    (proxy, task)
}
