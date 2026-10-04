mod batch;
mod capture;
mod files;
pub(crate) use capture::Collected;
mod manager;
mod media;
mod preparing;
mod process;
use manager::{Manager, Status};
use serde::Deserialize;
use stagemaster_execution_client::{Action, AudioOutput, MediaAction, OutputAction, Selection};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::Manager as _;
use tokio::sync::Mutex;

pub(crate) struct Service(Arc<Mutex<Manager>>);
pub(crate) struct EditorAudioGuard {
    _manager: tokio::sync::OwnedMutexGuard<Manager>,
    _file: std::fs::File,
}
impl Service {
    pub fn editor_audio(&self) -> Result<EditorAudioGuard, String> {
        let mut manager = self
            .0
            .clone()
            .try_lock_owned()
            .map_err(|_| "后台正在准备或操作，请稍后试听")?;
        let file = manager.reserve_editor_audio()?;
        Ok(EditorAudioGuard {
            _manager: manager,
            _file: file,
        })
    }
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
        audio_output: Option<AudioOutput>,
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
    Batch {
        host_id: String,
        revision: String,
        sources: Vec<String>,
        action: stagemaster_execution_client::BatchAction,
    },
    Output {
        host_id: String,
        revision: String,
        action: OutputAction,
    },
    Media {
        host_id: String,
        revision: String,
        group: String,
        generation: String,
        action: MediaAction,
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
        .clone()
        .try_lock_owned()
        .map_err(|_| "后台操作正在处理，请稍后重试")?;
    match request {
        Request::Snapshot {} => Ok(manager.poll().await),
        Request::Reconnect {} => Ok(manager.reconnect().await),
        Request::Prepare {
            generation,
            selection,
            audio_output,
        } => preparing::run(app, manager, generation, selection, audio_output).await,
        Request::Acquire { takeover } => manager.acquire(takeover).await,
        Request::Release {} => manager.release().await,
        Request::Apply {
            host_id,
            revision,
            source,
            action,
        } => manager.apply(&host_id, &revision, &source, action).await,
        Request::Batch {
            host_id,
            revision,
            sources,
            action,
        } => manager.batch(&host_id, &revision, &sources, action).await,
        Request::Output {
            host_id,
            revision,
            action,
        } => manager.output(&host_id, &revision, action).await,
        Request::Media {
            host_id,
            revision,
            group,
            generation,
            action,
        } => {
            manager
                .apply_media(&host_id, &revision, &group, &generation, action)
                .await
        }
        Request::Shutdown { host_id } => manager.shutdown(&host_id).await,
    }
}
