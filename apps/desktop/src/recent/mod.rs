//! Local navigation metadata. Never an authority to bypass project validation.
mod store;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

pub(crate) struct Service(store::Store);
impl Service {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self(store::Store(root))
    }
    pub(crate) fn resolve(&self, id: &str) -> Result<PathBuf, String> {
        self.0.resolve(id)
    }
    pub(crate) fn remember(&self, path: &Path, name: &str) -> Result<(), String> {
        self.0.remember(path, name)
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    List,
    Forget { id: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry {
    #[serde(flatten)]
    record: store::Entry,
    available: bool,
}

#[tauri::command]
pub(crate) async fn recent_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Vec<Entry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = &app.state::<Service>().0;
        if let Request::Forget { id } = request {
            store.forget(&id)?;
        }
        Ok(store
            .list()?
            .into_iter()
            .map(|record| Entry {
                available: std::fs::symlink_metadata(&record.path).is_ok_and(|m| m.is_file()),
                record,
            })
            .collect())
    })
    .await
    .map_err(|_| "最近工程操作未完成".to_string())?
}
