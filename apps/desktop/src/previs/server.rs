#[path = "editing.rs"]
mod editing;
use super::background::{Background, SharedBackground};
use super::protocol::{Frame, PlacementRequest, Source, Stamp};
use super::{SharedSession, Status};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use editing::{frame_response, scene_response};
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{Semaphore, oneshot};
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

const MAX_SCENE_BYTES: usize = 32 * 1024 * 1024;
type Changed = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct Activity {
    seen: Option<Instant>,
    problem: Option<String>,
}
struct Cached {
    version: u64,
    rig: stagemaster_previs::LightRig,
    scene: stagemaster_previs::Scene,
}
#[derive(Serialize)]
struct SceneResponse<'a> {
    #[serde(flatten)]
    stamp: Stamp,
    scene: &'a stagemaster_previs::Scene,
}
struct Context {
    shared: SharedSession,
    background: SharedBackground,
    bridge_id: String,
    authorization: String,
    // Lock held through the mutation: disable revokes even already-authenticated requests.
    active: Mutex<bool>,
    cache: Mutex<Option<Arc<Cached>>>,
    activity: Mutex<Activity>,
    workers: Arc<Semaphore>,
    changed: Changed,
}
pub(super) struct Server {
    context: Arc<Context>,
    address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: tauri::async_runtime::JoinHandle<()>,
}
impl Server {
    pub(super) fn configure_renderer(&self, command: &mut std::process::Command) {
        command
            .env("STAGEMASTER_PREVIS_URL", format!("http://{}", self.address))
            .env(
                "STAGEMASTER_PREVIS_AUTHORIZATION",
                &self.context.authorization,
            )
            .env("STAGEMASTER_PREVIS_SESSION", &self.context.bridge_id);
    }
    #[cfg(test)]
    pub(super) async fn start(shared: SharedSession, changed: Changed) -> Result<Self, String> {
        Self::with_background(shared, changed, SharedBackground::default()).await
    }
    pub(super) async fn with_background(
        shared: SharedSession,
        changed: Changed,
        background: SharedBackground,
    ) -> Result<Self, String> {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| "无法建立本机预演连接")?;
        let address = listener.local_addr().map_err(|_| "无法读取预演端口")?;
        let context = Arc::new(Context {
            shared,
            background,
            changed,
            bridge_id: uuid::Uuid::new_v4().to_string(),
            authorization: format!(
                "Bearer {}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            ),
            active: Mutex::new(true),
            cache: Mutex::new(None),
            activity: Mutex::new(Activity::default()),
            workers: Arc::new(Semaphore::new(2)),
        });
        let router = Router::new()
            .route("/v1/scene", get(scene))
            .route("/v1/frame", get(frame))
            .route("/v1/placement", post(placement))
            .layer(DefaultBodyLimit::max(8192))
            .layer(middleware::from_fn_with_state(context.clone(), authorize))
            .with_state(context.clone());
        let (shutdown, receiver) = oneshot::channel();
        let task_context = context.clone();
        let task = tauri::async_runtime::spawn(async move {
            let result = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
            if let Ok(mut active) = task_context.active.lock() {
                *active = false;
            }
            if result.is_err()
                && let Ok(mut activity) = task_context.activity.lock()
            {
                activity.problem = Some("预演连接已停止，请重新打开预演".into());
            }
        });
        Ok(Self {
            context,
            address,
            shutdown: Some(shutdown),
            task,
        })
    }
    pub(super) fn status(&self, source: Source, now: Instant) -> Status {
        let activity = self.context.activity.lock().ok();
        let enabled = self.context.active.lock().is_ok_and(|a| *a);
        Status {
            enabled,
            connected: enabled
                && activity.as_ref().and_then(|a| a.seen).is_some_and(|last| {
                    now.saturating_duration_since(last) < Duration::from_secs(2)
                }),
            port: Some(self.address.port()),
            viewer_url: None,
            source,
            problem: activity.and_then(|a| a.problem.clone()),
            background: None,
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if let Ok(mut active) = self.context.active.lock() {
            *active = false;
        }
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // No renderer work is allowed to keep the host alive after disabling the bridge.
        self.task.abort();
    }
}

#[derive(Debug)]
struct Failure(StatusCode, String);
impl Failure {
    fn busy() -> Self {
        Self(
            StatusCode::SERVICE_UNAVAILABLE,
            "工程正在处理其他操作，请稍后重试".into(),
        )
    }
    fn conflict(message: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, message.into())
    }
    fn invalid(message: impl Into<String>) -> Self {
        Self(StatusCode::UNPROCESSABLE_ENTITY, message.into())
    }
}
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({"message":self.1}))).into_response()
    }
}
async fn authorize(
    State(context): State<Arc<Context>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers();
    if headers.contains_key(header::ORIGIN)
        || !credential_matches(headers, &context.authorization)
        || !context.active.lock().is_ok_and(|a| *a)
    {
        return Failure(StatusCode::FORBIDDEN, "预演连接未授权或已关闭".into()).into_response();
    }
    let mut response = match tokio::time::timeout(Duration::from_secs(3), next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            Failure(StatusCode::REQUEST_TIMEOUT, "预演请求超时，请重试".into()).into_response()
        }
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}
fn credential_matches(headers: &HeaderMap, expected: &str) -> bool {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let Some(actual) = values.next() else {
        return false;
    };
    if values.next().is_some() || actual.as_bytes().len() != expected.len() {
        return false;
    }
    actual
        .as_bytes()
        .iter()
        .zip(expected.bytes())
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

async fn scene(State(context): State<Arc<Context>>) -> Result<Response, Failure> {
    if let Some(background) = background(&context)? {
        let bytes = serde_json::to_vec(&SceneResponse {
            stamp: Stamp::new(&context.bridge_id, background.revision),
            scene: &background.scene,
        })
        .map_err(|_| Failure::invalid("后台场地序列化失败"))?;
        if bytes.len() > MAX_SCENE_BYTES {
            return Err(Failure::invalid("场地预演数据超出 32 MiB 限制"));
        }
        return Ok(([(header::CONTENT_TYPE, "application/json")], bytes).into_response());
    }
    blocking(context, |context| scene_response(&context)).await
}
async fn frame(State(context): State<Arc<Context>>) -> Result<Json<Frame>, Failure> {
    if let Some(binding) = background(&context)? {
        let lights = match binding.lights().await {
            Ok(lights) => lights,
            Err(message) => {
                if let Ok(mut activity) = context.activity.lock() {
                    activity.problem = Some(message.clone());
                }
                return Err(Failure::conflict(message));
            }
        };
        let current = background(&context)?.ok_or_else(|| Failure::conflict("预演来源已变化"))?;
        if !Arc::ptr_eq(&current, &binding) {
            return Err(Failure::conflict("后台观察已变化"));
        }
        let mut activity = context.activity.lock().map_err(|_| Failure::busy())?;
        activity.seen = Some(Instant::now());
        activity.problem = None;
        return Ok(Json(Frame {
            stamp: Stamp::new(&context.bridge_id, binding.revision),
            source: Source::Background {
                host_id: binding.host_id.clone(),
            },
            status: "background",
            can_edit: false,
            lights,
        }));
    }
    blocking(context, |context| frame_response(&context)).await
}
fn background(context: &Context) -> Result<Option<Arc<Background>>, Failure> {
    let source = context
        .shared
        .try_lock()
        .map_err(|_| Failure::busy())?
        .previs_source();
    let Source::Background { host_id } = source else {
        return Ok(None);
    };
    let value = context
        .background
        .lock()
        .map_err(|_| Failure::busy())?
        .current
        .clone()
        .filter(|value| value.host_id == host_id)
        .ok_or_else(|| Failure::conflict("请重新连接后台三维来源"))?;
    Ok(Some(value))
}
async fn blocking<T: Send + 'static>(
    context: Arc<Context>,
    work: impl FnOnce(Arc<Context>) -> Result<T, Failure> + Send + 'static,
) -> Result<T, Failure> {
    let permit = context
        .workers
        .clone()
        .try_acquire_owned()
        .map_err(|_| Failure::busy())?;
    tokio::task::spawn_blocking(move || {
        // Keep the capacity until CPU work ends, including after HTTP timeout/disconnect.
        let _permit = permit;
        work(context)
    })
    .await
    .map_err(|_| Failure::busy())?
}
async fn placement(
    State(context): State<Arc<Context>>,
    request: Result<Json<PlacementRequest>, JsonRejection>,
) -> Result<Json<Stamp>, Failure> {
    let Json(request) = request.map_err(|_| Failure::invalid("灯位请求格式或长度不正确"))?;
    blocking(context, move |context| place_response(&context, request)).await
}
fn place_response(context: &Context, request: PlacementRequest) -> Result<Json<Stamp>, Failure> {
    if request.bridge_id != context.bridge_id {
        return Err(Failure::conflict("预演会话已更换"));
    }
    let stamp = {
        let active = context.active.lock().map_err(|_| Failure::busy())?;
        if !*active {
            return Err(Failure::conflict("预演连接已关闭"));
        }
        let revision = context
            .shared
            .try_lock()
            .map_err(|_| Failure::busy())?
            .previs_place(request)
            .map_err(Failure::conflict)?;
        Stamp::new(&context.bridge_id, revision)
    };
    (context.changed)();
    Ok(Json(stamp))
}
