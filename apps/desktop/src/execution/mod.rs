mod files;
mod manager;
mod process;
use crate::previs::SharedSession;
use manager::{Manager, Status};
use serde::Deserialize;
use stagemaster_execution_client::{Action, Selection};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::Manager as _;
use tokio::sync::Mutex;

pub(crate) struct Service(Arc<Mutex<Manager>>);
impl Service {
    pub fn discovery_path(&self) -> Result<PathBuf, String> {
        self.0
            .try_lock()
            .map_err(|_| "后台操作正在处理，请稍后重试")?
            .discovery_path()
    }
    pub fn new(app: &tauri::AppHandle, root: PathBuf) -> Self {
        // An unavailable bundled program is reported on preparation, not an editor startup failure.
        let binary = process::binary(app).unwrap_or_else(|_| PathBuf::new());
        let inner = Arc::new(Mutex::new(Manager::new(root, binary)));
        let weak = Arc::downgrade(&inner);
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let Some(inner) = weak.upgrade() else {
                    break;
                };
                if let Ok(mut manager) = inner.try_lock() {
                    manager.poll().await;
                }
            }
        });
        Self(inner)
    }
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Request {
    Snapshot {},
    Prepare {
        generation: u32,
        selection: Vec<Selection>,
    },
    Reconnect {},
    Acquire {
        takeover: bool,
    },
    Release {},
    Apply {
        host_id: String,
        revision: String,
        source: String,
        action: Action,
    },
    Shutdown {
        host_id: String,
    },
}
#[tauri::command]
pub(crate) async fn execution_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Status, String> {
    let service = app.state::<Service>();
    let mut manager = service
        .0
        .try_lock()
        .map_err(|_| "后台操作正在处理，请稍后重试")?;
    match request {
        Request::Snapshot {} => Ok(manager.poll().await),
        Request::Reconnect {} => Ok(manager.reconnect().await),
        Request::Prepare {
            generation,
            selection,
        } => {
            let document = app
                .state::<SharedSession>()
                .lock()
                .map_err(|_| "工程会话不可用")?
                .check_snapshot(generation)?;
            manager.prepare(document, selection).await
        }
        Request::Acquire { takeover } => manager.acquire(takeover).await,
        Request::Release {} => manager.release().await,
        Request::Apply {
            host_id,
            revision,
            source,
            action,
        } => manager.apply(&host_id, &revision, &source, action).await,
        Request::Shutdown { host_id } => manager.shutdown(&host_id).await,
    }
}
