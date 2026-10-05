//! Desktop transport boundary. The renderer never owns a document, show clock or output lease.
mod background;
mod background_read_failure;
pub(crate) mod protocol;
mod renderer;
mod renderer_component;
mod renderer_local;
mod renderer_paths;
mod selection;
mod server;
mod session_access;

use crate::session::Session;
use protocol::{Request, Source};
use serde::Serialize;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;
use tauri::{Emitter, Manager};

pub(crate) type SharedSession = Arc<Mutex<Session>>;
#[derive(Default)]
pub(crate) struct Bridge {
    runtime: tokio::sync::Mutex<Runtime>,
    closing: AtomicBool,
    background: background::SharedBackground,
}

#[derive(Default)]
struct Runtime {
    server: Option<server::Server>,
    renderer: Option<renderer::Renderer>,
    problem: Option<String>,
}
impl Runtime {
    fn stop(&mut self) {
        self.server.take();
        self.renderer.take();
    }
    fn observe_exit(&mut self) {
        if self
            .renderer
            .as_mut()
            .is_some_and(renderer::Renderer::exited)
        {
            self.stop();
            self.problem = Some("三维预演已关闭，可重新打开".into());
        }
    }
}
impl Bridge {
    pub(crate) fn begin_shutdown(&self) {
        self.closing.store(true, Ordering::Release);
    }
    fn check_open(&self) -> Result<(), String> {
        if self.closing.load(Ordering::Acquire) {
            Err("应用正在退出".into())
        } else {
            Ok(())
        }
    }
    async fn access(&self) -> Result<tokio::sync::MutexGuard<'_, Runtime>, String> {
        self.check_open()?;
        let runtime = self.runtime.lock().await;
        self.check_open()?;
        Ok(runtime)
    }
    pub(crate) async fn close(&self) {
        self.begin_shutdown();
        self.runtime.lock().await.stop();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    enabled: bool,
    connected: bool,
    port: Option<u16>,
    viewer_url: Option<String>,
    source: Source,
    problem: Option<String>,
    background: Option<background::Summary>,
}

#[tauri::command]
pub(crate) async fn previs_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Status, String> {
    let state = app.state::<Bridge>();
    state.check_open()?;
    let shared = app.state::<SharedSession>().inner().clone();
    let mut runtime = state.access().await?;
    selection::apply(&app, &state, &shared, &request).await?;
    runtime.observe_exit();
    match request {
        Request::Enable
            if runtime
                .server
                .as_ref()
                .is_none_or(|server| !server.status(Source::Defaults, Instant::now()).enabled) =>
        {
            session_access::access(&shared, |session| {
                let generation = session.previs_revision().generation;
                session.set_previs_editing(generation, false)
            })
            .await?;
            runtime.stop();
            let notify_app = app.clone();
            let server = server::Server::with_background(
                shared.clone(),
                Arc::new(move || {
                    let _ = notify_app.emit("project-updated", ());
                }),
                state.background.clone(),
            )
            .await?;
            let renderer = renderer::Renderer::start(&app, &server)?;
            runtime.server = Some(server);
            runtime.renderer = Some(renderer);
            runtime.problem = None;
        }
        Request::Disable => {
            runtime.stop();
            runtime.problem = None;
        }
        _ => {}
    }
    let source = session_access::access(&shared, |session| Ok(session.previs_source())).await?;
    let mut status = runtime.server.as_ref().map_or(
        Status {
            enabled: false,
            connected: false,
            port: None,
            viewer_url: None,
            source: source.clone(),
            problem: runtime.problem.clone(),
            background: None,
        },
        |server| server.status(source, Instant::now()),
    );
    status.viewer_url = runtime
        .renderer
        .as_ref()
        .map(|renderer| renderer.viewer_url().to_string());
    if let Source::Background { host_id } = &status.source {
        status.background = state
            .background
            .lock()
            .map_err(|_| "后台观察不可用")?
            .current
            .as_ref()
            .filter(|value| &value.host_id == host_id)
            .map(|value| value.summary());
    }
    Ok(status)
}

#[cfg(test)]
#[path = "shutdown_tests.rs"]
mod shutdown_tests;
