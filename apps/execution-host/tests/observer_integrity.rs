mod support;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Method, Request},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use stagemaster_execution_client::Reader;
use std::{
    fs,
    sync::{Arc, Mutex},
    time::Duration,
};
use support::*;

struct Proxy {
    base: String,
    token: String,
    http: reqwest::Client,
    replacement: Mutex<Option<(&'static str, Value)>>,
}
async fn forward(State(proxy): State<Arc<Proxy>>, request: Request<Body>) -> Response {
    assert_eq!(request.method(), Method::GET, "观察者不得提交控制操作");
    assert_eq!(request.headers()["authorization"], proxy.token);
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
fn reader_rejects_wrong_identity_partial_frames_stale_observation_and_changed_project() {
    runtime().block_on(async {
        let mut h = Harness::start_group();
        let proxy = Arc::new(Proxy {
            base: h.discovery["url"]
                .as_str()
                .unwrap()
                .split("/v2/")
                .next()
                .unwrap()
                .into(),
            token: format!("Bearer {}", h.discovery["readToken"].as_str().unwrap()),
            http: client(),
            replacement: Mutex::new(None),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = Router::new().fallback(forward).with_state(proxy.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let path = h.directory.path().join("run/observer.json");
        let mut discovery = h.discovery.clone();
        discovery["url"] = format!(
            "http://{address}/v2/{}",
            discovery["hostId"].as_str().unwrap()
        )
        .into();
        fs::write(&path, serde_json::to_vec(&discovery).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let mut reader = Reader::open(&path).await.unwrap();
        reader.project().await.unwrap();
        *proxy.replacement.lock().unwrap() = Some(("/project", json!({"altered":true})));
        assert!(reader.project().await.is_err());
        let valid = ok(h.get("/state")).await;
        for (pointer, value) in [
            ("/phase", json!("faulted")),
            ("/fault", json!("failure")),
            ("/snapshot/state/fault", json!(true)),
            ("/snapshot/state/output", json!(null)),
            ("/snapshot/state/output/percent", json!(101)),
            ("/snapshot/state/output/blackout", json!("true")),
            ("/snapshot/state/boot", json!("another-host")),
            ("/snapshot/state/layout", json!("another-layout")),
            ("/snapshot/frame/boot", json!("another-host")),
            ("/snapshot/frame/layout", json!("another-layout")),
            ("/snapshot/frame/revision", json!("9999")),
            ("/snapshot/frame/kind", json!("physicalFeedback")),
            ("/snapshot/frame/universe", json!(0)),
            ("/snapshot/frame/slots", json!([0, 1])),
            ("/snapshot/frame/sampledMs", json!("18446744073709551615")),
            ("/snapshot/state/observedMs", json!("18446744073709551615")),
            ("/snapshot/cycles", json!("01")),
        ] {
            let mut invalid = valid.clone();
            *invalid.pointer_mut(pointer).unwrap() = value;
            *proxy.replacement.lock().unwrap() = Some(("/state", invalid));
            assert!(reader.sample().await.is_err(), "未拒绝：{pointer}");
        }
        *proxy.replacement.lock().unwrap() = Some(("/state", valid));
        reader.sample().await.unwrap();
        tokio::time::sleep(Duration::from_millis(2050)).await;
        assert!(reader.sample().await.is_err(), "重复读取不能刷新旧帧有效期");
        *proxy.replacement.lock().unwrap() = None;
        reader.sample().await.unwrap();
        assert!(h.state().await["owner"].is_null());
        task.abort();
        h.close().await;
    });
}
