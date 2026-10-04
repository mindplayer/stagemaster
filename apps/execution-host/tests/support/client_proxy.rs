use super::{Harness, client};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

// Owned loopback fault injection. It forwards to the same real host and never logs credentials.
pub struct Proxy {
    base: String,
    http: reqwest::Client,
    pub corrupt: AtomicBool,
    pub reject_commands: AtomicBool,
    pub reject_observations: AtomicBool,
    pub submissions: AtomicUsize,
    pub forwarded_commands: AtomicUsize,
    pub receipt_reads: AtomicUsize,
    pub old_observation: Mutex<Option<Value>>,
}
async fn forward(State(proxy): State<Arc<Proxy>>, request: Request<Body>) -> Response {
    let path = request.uri().path();
    let command = path.ends_with("/commands");
    let observation = path.ends_with("/state");
    if path.contains("/receipts/") {
        proxy.receipt_reads.fetch_add(1, Ordering::SeqCst);
    }
    if command {
        proxy.submissions.fetch_add(1, Ordering::SeqCst);
        if proxy.reject_commands.load(Ordering::SeqCst) {
            return busy();
        }
    }
    if observation {
        if proxy.reject_observations.load(Ordering::SeqCst) {
            return busy();
        }
        if let Some(old) = proxy.old_observation.lock().unwrap().clone() {
            return axum::Json(old).into_response();
        }
    }
    let builder = proxy
        .http
        .request(request.method().clone(), format!("{}{path}", proxy.base))
        .headers(request.headers().clone());
    let body = to_bytes(request.into_body(), 8192).await.unwrap();
    if command {
        proxy.forwarded_commands.fetch_add(1, Ordering::SeqCst);
    }
    let response = builder.body(body).send().await.unwrap();
    let status = response.status();
    let bytes = response.bytes().await.unwrap();
    if command && proxy.corrupt.swap(false, Ordering::SeqCst) {
        // Real admission has already happened. Only its reply is damaged.
        return (StatusCode::OK, "{truncated").into_response();
    }
    (status, bytes).into_response()
}
fn busy() -> Response {
    (StatusCode::SERVICE_UNAVAILABLE, "injected busy response").into_response()
}
pub async fn install(h: &Harness, path: &Path) -> (Arc<Proxy>, tokio::task::JoinHandle<()>) {
    let actual = h.discovery["url"].as_str().unwrap();
    let proxy = Arc::new(Proxy {
        base: actual.split("/v2/").next().unwrap().to_owned(),
        http: client(),
        corrupt: AtomicBool::new(false),
        reject_commands: AtomicBool::new(false),
        reject_observations: AtomicBool::new(false),
        submissions: AtomicUsize::new(0),
        forwarded_commands: AtomicUsize::new(0),
        receipt_reads: AtomicUsize::new(0),
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
