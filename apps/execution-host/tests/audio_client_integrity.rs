mod support;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Method, Request},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use stagemaster_execution_client::{Client, Reader};
use std::{
    fs,
    sync::{Arc, Mutex},
};
use support::*;
struct Proxy {
    base: String,
    http: reqwest::Client,
    replacement: Mutex<Option<(&'static str, Value)>>,
}
async fn forward(State(proxy): State<Arc<Proxy>>, request: Request<Body>) -> Response {
    assert_eq!(request.method(), Method::GET);
    let path = request.uri().path();
    if let Some((suffix, value)) = proxy.replacement.lock().unwrap().as_ref()
        && path.ends_with(suffix)
    {
        return axum::Json(value.clone()).into_response();
    }
    let response = proxy
        .http
        .get(format!("{}{path}", proxy.base))
        .headers(request.headers().clone())
        .send()
        .await
        .unwrap();
    (response.status(), response.bytes().await.unwrap()).into_response()
}
#[test]
fn music_catalog_and_state_must_agree_for_controllers_and_readonly_renderers() {
    runtime().block_on(async {
        let mut h = Harness::prepared(|p| Some(audio::write(p)));
        let proxy = Arc::new(Proxy {
            base: h.discovery["url"]
                .as_str()
                .unwrap()
                .split("/v2/")
                .next()
                .unwrap()
                .into(),
            http: client(),
            replacement: Mutex::new(None),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = Router::new().fallback(forward).with_state(proxy.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let path = h.directory.path().join("run/discovery.json");
        let mut discovery = h.discovery.clone();
        discovery["url"] = format!(
            "http://{address}/v2/{}",
            discovery["hostId"].as_str().unwrap()
        )
        .into();
        fs::write(&path, serde_json::to_vec(&discovery).unwrap()).unwrap();
        let valid = ok(h.get("/source")).await;
        for (pointer, value) in [
            ("/audio", Value::Null),
            ("/audio/group", json!(group::id(99))),
            ("/audio/durationMs", json!(0)),
            ("/audio/output", json!("unknown")),
            ("/audio/seekIncludesEnd", json!(true)),
            ("/capabilities", json!([])),
            ("/sources/1/selection/kind", json!("manual")),
        ] {
            let mut invalid = valid.clone();
            *invalid.pointer_mut(pointer).unwrap() = value;
            *proxy.replacement.lock().unwrap() = Some(("/source", invalid));
            assert!(
                Client::open(&path).await.is_err(),
                "controller accepted {pointer}"
            );
            assert!(
                Reader::open(&path).await.is_err(),
                "reader accepted {pointer}"
            );
        }
        *proxy.replacement.lock().unwrap() = None;
        let mut reader = Reader::open(&path).await.unwrap();
        let valid = ok(h.get("/state")).await;
        for (pointer, value) in [
            ("/snapshot/state/audio", Value::Null),
            ("/snapshot/state/audio/output", json!("systemDefault")),
            ("/snapshot/state/audio/durationMs", json!(5001)),
            ("/snapshot/state/audio/positionMs", json!(5001)),
            ("/snapshot/state/audio/frames", json!("01")),
            ("/snapshot/state/audio/instance", json!("0")),
            ("/snapshot/state/media/0/id", json!(group::id(99))),
            ("/snapshot/state/media/0/generation", json!("-1")),
            (
                "/snapshot/state/media/0/control",
                json!({"request":"0","status":"pending","problem":null}),
            ),
            ("/snapshot/state/media", json!([])),
        ] {
            let mut invalid = valid.clone();
            *invalid.pointer_mut(pointer).unwrap() = value;
            *proxy.replacement.lock().unwrap() = Some(("/state", invalid));
            assert!(
                Client::open(&path).await.is_err(),
                "controller accepted {pointer}"
            );
            assert!(reader.sample().await.is_err(), "reader accepted {pointer}");
        }
        *proxy.replacement.lock().unwrap() = None;
        Client::open(&path).await.unwrap();
        reader.sample().await.unwrap();
        assert!(h.state().await["owner"].is_null());
        task.abort();
        h.close().await;
    });
}
