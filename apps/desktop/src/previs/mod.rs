//! Desktop transport boundary. The renderer never owns a document, show clock or output lease.
pub(crate) mod protocol;
mod server;

use crate::session::Session;
use protocol::{Request, Source};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, Manager};

pub(crate) type SharedSession = Arc<Mutex<Session>>;
#[derive(Default)]
pub(crate) struct Bridge(tokio::sync::Mutex<Option<server::Server>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    enabled: bool,
    connected: bool,
    port: Option<u16>,
    source: Source,
    problem: Option<String>,
}

#[tauri::command]
pub(crate) async fn previs_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Status, String> {
    let shared = app.state::<SharedSession>().inner().clone();
    match request {
        Request::Source {
            generation,
            ref source,
        } => shared
            .try_lock()
            .map_err(|_| "工程正在处理其他操作")?
            .set_previs_source(generation, source.clone())?,
        Request::Editing {
            generation,
            allowed,
        } => shared
            .try_lock()
            .map_err(|_| "工程正在处理其他操作")?
            .set_previs_editing(generation, allowed)?,
        Request::Status | Request::Enable | Request::Disable => {}
    }
    let state = app.state::<Bridge>();
    let mut bridge = state.0.lock().await;
    match request {
        Request::Enable
            if bridge
                .as_ref()
                .is_none_or(|server| !server.status(Source::Defaults, Instant::now()).enabled) =>
        {
            {
                let mut session = shared.try_lock().map_err(|_| "工程正在处理其他操作")?;
                let generation = session.previs_revision().generation;
                session.set_previs_editing(generation, false)?;
            }
            bridge.take();
            let notify_app = app.clone();
            *bridge = Some(
                server::Server::start(
                    shared.clone(),
                    Arc::new(move || {
                        let _ = notify_app.emit("project-updated", ());
                    }),
                )
                .await?,
            );
        }
        Request::Disable => {
            bridge.take();
        }
        _ => {}
    }
    let source = shared
        .try_lock()
        .map_err(|_| "工程正在处理其他操作")?
        .previs_source();
    Ok(bridge.as_ref().map_or(
        Status {
            enabled: false,
            connected: false,
            port: None,
            source: source.clone(),
            problem: None,
        },
        |server| server.status(source, Instant::now()),
    ))
}
