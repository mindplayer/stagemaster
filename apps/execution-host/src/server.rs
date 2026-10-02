use crate::{
    bounded_io::BoundedListener,
    directory::Directory,
    preparation::Prepared,
    projection,
    service::Service,
    wire::{Decimal, Failure, Input, RecordView},
};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Discovery {
    protocol: u8,
    host_id: String,
    url: String,
    read_token: String,
    control_token: String,
}
struct Context {
    service: Arc<Service>,
    read_header: String,
    control_header: String,
}
fn token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
fn matches(actual: &[u8], expected: &str) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.bytes())
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}
pub(crate) async fn serve(prepared: Prepared, mut directory: Directory) -> Result<(), String> {
    let service = Service::new(prepared);
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|e| e.to_string())?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    let discovery = Discovery {
        protocol: 1,
        host_id: service.id.to_string(),
        url: format!("http://{address}/v1/{}", service.id),
        read_token: token(),
        control_token: token(),
    };
    let context = Arc::new(Context {
        service: service.clone(),
        read_header: format!("Bearer {}", discovery.read_token),
        control_header: format!("Bearer {}", discovery.control_token),
    });
    let router = Router::new()
        .route("/v1/{host}/source", get(source))
        .route("/v1/{host}/state", get(state))
        .route("/v1/{host}/sessions", post(create_session))
        .route("/v1/{host}/sessions/{session}/commands", post(submit))
        .route(
            "/v1/{host}/sessions/{session}/receipts/{serial}",
            get(receipt),
        )
        .route("/v1/{host}/shutdown", post(shutdown))
        .layer(DefaultBodyLimit::max(8192))
        .layer(middleware::from_fn_with_state(context.clone(), authorize))
        .with_state(context);
    directory.publish(&discovery).map_err(|e| e.to_string())?;
    println!("独立执行宿主已就绪：{}（仅软件执行）", service.id);
    let (completed, mut stopped) = tokio::sync::oneshot::channel();
    let stopping = service.clone();
    let result = axum::serve(BoundedListener::new(listener), router)
        .with_graceful_shutdown(async move {
            stopping.shutdown.notified().await;
            let _ = completed.send(stopping.join().await);
        })
        .await;
    if let Ok(result) = stopped.try_recv() {
        result?;
    } else {
        service.begin_shutdown().map_err(|e| e.2.to_string())?;
        service.join().await?;
    }
    result.map_err(|e| e.to_string())
}
async fn authorize(
    State(context): State<Arc<Context>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let header = values.next();
    let unique = header.is_some() && values.next().is_none();
    let control = header.is_some_and(|h| matches(h.as_bytes(), &context.control_header));
    let read = header.is_some_and(|h| matches(h.as_bytes(), &context.read_header));
    let authorized = unique
        && !request.headers().contains_key(header::ORIGIN)
        && (control || (read && request.method() == axum::http::Method::GET));
    let response = if !authorized {
        Failure(StatusCode::FORBIDDEN, "forbidden", "此请求没有所需权限").into_response()
    } else if request.uri().path().split('/').nth(2)
        != Some(context.service.id.to_string().as_str())
    {
        Failure(StatusCode::CONFLICT, "hostChanged", "执行宿主身份已更换").into_response()
    } else if !context.service.available() {
        Failure::closed().into_response()
    } else {
        match tokio::time::timeout(Duration::from_secs(3), limited(request, next)).await {
            Ok(response) => response,
            Err(_) => Failure(
                StatusCode::REQUEST_TIMEOUT,
                "timeout",
                "请求等待超时，已接纳的操作须查询原回执",
            )
            .into_response(),
        }
    };
    let mut response = response;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}
async fn limited(request: Request<Body>, next: Next) -> Response {
    // DefaultBodyLimit alone only protects extractors which actually read a body.
    // Apply the budget before every route, including session creation and shutdown.
    let (parts, body) = request.into_parts();
    match axum::body::to_bytes(body, 8192).await {
        Ok(bytes) => {
            next.run(Request::from_parts(parts, Body::from(bytes)))
                .await
        }
        Err(_) => Failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "bodyLimit",
            "请求超过 8 KiB 或无法完整读取",
        )
        .into_response(),
    }
}
async fn source(State(context): State<Arc<Context>>) -> Json<Value> {
    Json(context.service.source.clone())
}
async fn state(State(context): State<Arc<Context>>) -> Result<Json<Value>, Failure> {
    Ok(Json(projection::observation(
        &context
            .service
            .observer
            .read()
            .map_err(|_| Failure::busy())?,
    )))
}
async fn create_session(State(context): State<Arc<Context>>) -> Result<Json<Value>, Failure> {
    Ok(Json(
        json!({"hostId":context.service.id.to_string(),"sessionId":context.service.create_session()?.to_string(),"nextSerial":"1"}),
    ))
}
fn session_id(value: &str) -> Result<Uuid, Failure> {
    Uuid::parse_str(value).map_err(|_| Failure::invalid())
}
async fn submit(
    State(context): State<Arc<Context>>,
    Path((_, session)): Path<(String, String)>,
    input: Result<Json<Input>, JsonRejection>,
) -> Result<Json<RecordView>, Failure> {
    let Json(input) = input.map_err(|e| {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Failure(
                StatusCode::PAYLOAD_TOO_LARGE,
                "bodyLimit",
                "请求超过 8 KiB 限制",
            )
        } else {
            Failure::invalid()
        }
    })?;
    context
        .service
        .submit(session_id(&session)?, input)
        .map(Json)
}
async fn receipt(
    State(context): State<Arc<Context>>,
    Path((_, session, serial)): Path<(String, String, String)>,
) -> Result<Json<RecordView>, Failure> {
    let serial = Decimal::try_from(serial).map_err(|_| Failure::invalid())?;
    context
        .service
        .receipt(session_id(&session)?, serial.0)
        .map(Json)
}
async fn shutdown(State(context): State<Arc<Context>>) -> Result<Json<Value>, Failure> {
    context.service.begin_shutdown()?;
    Ok(Json(json!({"status":"stopping"})))
}
