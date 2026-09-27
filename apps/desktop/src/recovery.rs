//! Desktop lifecycle policy over the portable recovery store. No playback state is saved.
use serde::{Deserialize, Serialize};
use stagemaster_project::Document;
use stagemaster_project_store::{
    RecoveryCandidate, RecoveryCatalog, RecoverySession, RecoveryStore,
};
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    pub state: Protection,
    pub captured_at_ms: Option<u64>,
    pub problem: Option<String>,
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Protection {
    #[default]
    Clean,
    Protected,
    Unprotected,
}
pub(crate) struct Checkpoint {
    pub generation: u32,
    pub document: Option<Document>,
    pub source_file: Option<String>,
}
pub(crate) struct Service {
    /// All project mutations, including their checkpoint/retirement, have a single order.
    pub operations: Mutex<()>,
    coordinator: Mutex<Coordinator>,
}
struct Coordinator {
    root: PathBuf,
    session: Option<RecoverySession>,
    claimed: Option<RecoveryCandidate>,
    synchronized: Option<u32>,
    status: Status,
}
impl Service {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            operations: Mutex::new(()),
            coordinator: Mutex::new(Coordinator {
                root,
                session: None,
                claimed: None,
                synchronized: None,
                status: Status::default(),
            }),
        }
    }
    pub(crate) fn status_after(&self, checkpoint: Checkpoint) -> Status {
        match self.coordinator.lock() {
            Ok(mut coordinator) => coordinator.synchronize(checkpoint),
            Err(_) => Status {
                state: Protection::Unprotected,
                problem: Some("恢复服务发生错误，请先手动保存工程".into()),
                captured_at_ms: None,
            },
        }
    }
    pub(crate) fn retire(&self) -> Result<(), String> {
        self.coordinator
            .lock()
            .map_err(|_| "恢复服务发生错误")?
            .retire()
    }
    fn store(&self) -> Result<RecoveryStore, String> {
        let root = self
            .coordinator
            .lock()
            .map_err(|_| "恢复服务发生错误")?
            .root
            .clone();
        RecoveryStore::open(&root)
    }
    pub(crate) fn claim(&self, id: &str, token: &str) -> Result<RecoveryCandidate, String> {
        self.store()?.claim(id, token)
    }
    pub(crate) fn take_claim(&self, claim: RecoveryCandidate) -> Result<(), String> {
        let mut coordinator = self.coordinator.lock().map_err(|_| "恢复服务发生错误")?;
        // The current project must have been retired before installing another claim.
        if coordinator.claimed.is_some() {
            return Err("上一个恢复副本尚未处理，请先保存工程".into());
        }
        coordinator.claimed = Some(claim);
        coordinator.synchronized = None;
        Ok(())
    }
}
impl Coordinator {
    fn ensure_session(&mut self) -> Result<&mut RecoverySession, String> {
        if self.session.is_none() {
            self.session = Some(RecoveryStore::open(&self.root)?.begin()?);
        }
        self.session.as_mut().ok_or_else(|| "恢复会话不可用".into())
    }
    fn retire_claim(&mut self) -> Result<(), String> {
        if let Some(claim) = self.claimed.as_mut() {
            claim.discard()?;
        }
        self.claimed = None;
        Ok(())
    }
    fn retire(&mut self) -> Result<(), String> {
        self.synchronized = None;
        if let Some(session) = self.session.as_mut() {
            session.clear()?;
        }
        self.retire_claim()?;
        self.status = Status::default();
        Ok(())
    }
    fn synchronize(&mut self, checkpoint: Checkpoint) -> Status {
        if self.synchronized == Some(checkpoint.generation) {
            return self.status.clone();
        }
        let result = if let Some(document) = checkpoint.document {
            let now = now_ms();
            self.ensure_session()
                .and_then(|session| session.checkpoint(&document, checkpoint.source_file, now))
                .map(|()| {
                    self.status = Status {
                        state: Protection::Protected,
                        captured_at_ms: Some(now),
                        problem: None,
                    };
                    // A failed retirement may leave a duplicate, but the new complete copy
                    // is already durable. Do not falsely report the editing snapshot lost.
                    if let Err(error) = self.retire_claim() {
                        self.status.problem =
                            Some(format!("恢复点已更新，但旧副本未清理：{error}"));
                    }
                })
        } else {
            self.retire()
        };
        match result {
            Ok(()) if self.status.problem.is_none() => {
                self.synchronized = Some(checkpoint.generation);
            }
            Ok(()) => self.synchronized = None,
            Err(error) => {
                self.synchronized = None;
                self.status.state = Protection::Unprotected;
                self.status.problem = Some(error);
            }
        }
        self.status.clone()
    }
}
fn now_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(9_007_199_254_740_991)
}
pub(crate) fn directory(app: &tauri::App) -> Result<PathBuf, tauri::Error> {
    if cfg!(debug_assertions) {
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/recovery"))
    } else {
        Ok(app.path().app_local_data_dir()?.join("recovery"))
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    List,
    Discard { id: String, token: String },
}
#[tauri::command]
pub(crate) async fn recovery_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<RecoveryCatalog, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = app.state::<Service>().store()?;
        if let Request::Discard { id, token } = request {
            store.discard(&id, &token)?;
        }
        store.list(now_ms())
    })
    .await
    .map_err(|_| "恢复目录操作未完成".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> tempfile::TempDir {
        tempfile::tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tmp")).unwrap()
    }
    fn point(generation: u32) -> Checkpoint {
        Checkpoint {
            generation,
            document: Some(Document::new("未保存工程").unwrap()),
            source_file: None,
        }
    }
    #[test]
    fn unchanged_generation_does_not_rewrite_and_clean_document_retires_snapshot() {
        let dir = directory();
        let service = Service::new(dir.path().to_owned());
        assert!(matches!(
            service.status_after(point(1)).state,
            Protection::Protected
        ));
        let store = service.store().unwrap();
        let first = store.list(now_ms()).unwrap().entries.remove(0);
        service.status_after(point(1));
        assert_eq!(store.list(now_ms()).unwrap().entries[0].token, first.token);
        assert!(matches!(
            service
                .status_after(Checkpoint {
                    generation: 2,
                    document: None,
                    source_file: None
                })
                .state,
            Protection::Clean
        ));
        assert!(store.list(now_ms()).unwrap().entries.is_empty());
        service.status_after(point(3));
        assert_eq!(store.list(now_ms()).unwrap().entries.len(), 1);
        service.retire().unwrap();
        assert!(store.list(now_ms()).unwrap().entries.is_empty());
    }
    #[test]
    fn claimed_original_survives_failed_checkpoint_and_retires_only_after_retry() {
        let dir = directory();
        let store = RecoveryStore::open(dir.path()).unwrap();
        let mut original = store.begin().unwrap();
        original
            .checkpoint(&Document::new("恢复来源").unwrap(), None, 1)
            .unwrap();
        drop(original);
        let entry = store.list(1).unwrap().entries.remove(0);
        let service = Service::new(dir.path().to_owned());
        service
            .take_claim(store.claim(&entry.id, &entry.token).unwrap())
            .unwrap();
        let mut others = Vec::new();
        for _ in 1..stagemaster_project_store::MAX_RECOVERY_RECORDS {
            let mut s = store.begin().unwrap();
            s.checkpoint(&Document::new("其他未保存内容").unwrap(), None, 1)
                .unwrap();
            others.push(s);
        }
        let status = service.status_after(point(1));
        assert!(matches!(status.state, Protection::Unprotected));
        assert!(status.problem.is_some());
        assert!(
            dir.path()
                .join(format!("{}.recovery.json", entry.id))
                .exists()
        );
        assert_eq!(
            store
                .list(2)
                .unwrap()
                .entries
                .iter()
                .find(|e| e.id == entry.id)
                .unwrap()
                .state,
            stagemaster_project_store::RecoveryState::Active
        );
        others[0].clear().unwrap();
        assert!(matches!(
            service.status_after(point(1)).state,
            Protection::Protected
        ));
        assert!(
            !dir.path()
                .join(format!("{}.recovery.json", entry.id))
                .exists()
        );
        service.retire().unwrap();
    }
    #[test]
    fn unavailable_storage_is_visible_and_can_retry_without_losing_content() {
        let dir = directory();
        let root = dir.path().join("blocked");
        std::fs::write(&root, b"external").unwrap();
        let service = Service::new(root.clone());
        let status = service.status_after(point(1));
        assert!(matches!(status.state, Protection::Unprotected));
        assert!(status.problem.is_some());
        assert_eq!(std::fs::read(&root).unwrap(), b"external");
        std::fs::remove_file(&root).unwrap();
        assert!(matches!(
            service.status_after(point(1)).state,
            Protection::Protected
        ));
    }
}
