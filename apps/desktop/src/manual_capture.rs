//! One bounded, expiring review ticket; applying uses the existing project history queue.
use serde::{Deserialize, Serialize};
use stagemaster_project::{
    ManualMergeSummary, ManualSceneCapture, ManualSceneMerge, ManualSceneReading,
};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::Manager;

const TTL: Duration = Duration::from_mins(5);
#[derive(Default)]
pub(crate) struct Service(Mutex<Option<Pending>>);
struct Pending {
    generation: u32,
    token: String,
    created: Instant,
    capture: ManualSceneCapture,
    merge: Option<ManualSceneMerge>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Captured {
    generation: u32,
    token: String,
    source_name: String,
    revision: String,
    readings: Vec<ManualSceneReading>,
    fixtures: Vec<stagemaster_execution_client::ManualFixture>,
    merge: Option<ManualMergeSummary>,
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Request {
    Capture {
        generation: u32,
        host_id: String,
        source: String,
        selected: Option<Vec<String>>,
        scene_id: Option<String>,
    },
    Cancel {
        token: String,
    },
}
impl Service {
    fn prepare(
        &self,
        generation: u32,
        collected: crate::execution::Collected,
        merge: Option<ManualSceneMerge>,
    ) -> Result<Captured, String> {
        let token = uuid::Uuid::new_v4().to_string();
        let response = Captured {
            generation,
            token: token.clone(),
            source_name: collected.source_name,
            revision: collected.revision,
            readings: collected.capture.readings().to_vec(),
            fixtures: collected.fixtures,
            merge: merge.as_ref().map(|m| m.summary().clone()),
        };
        *self.0.lock().map_err(|_| "手动记录会话不可用")? = Some(Pending {
            generation,
            token,
            created: Instant::now(),
            capture: collected.capture,
            merge,
        });
        Ok(response)
    }
    fn cancel(&self, token: &str) -> Result<(), String> {
        let mut pending = self.0.lock().map_err(|_| "手动记录会话不可用")?;
        if pending.as_ref().is_some_and(|p| p.token == token) {
            *pending = None;
        }
        Ok(())
    }
    pub(crate) fn apply(
        &self,
        generation: u32,
        token: &str,
        apply: impl FnOnce(&ManualSceneCapture) -> Result<(), String>,
    ) -> Result<(), String> {
        self.with_pending(generation, token, |p| {
            if p.merge.is_some() {
                return Err("此记录用于合并场景，请重新采集".into());
            }
            apply(&p.capture)
        })
    }
    pub(crate) fn apply_merge(
        &self,
        generation: u32,
        token: &str,
        apply: impl FnOnce(&ManualSceneMerge) -> Result<(), String>,
    ) -> Result<(), String> {
        self.with_pending(generation, token, |p| {
            apply(p.merge.as_ref().ok_or("此记录用于新场景，请重新采集")?)
        })
    }
    fn with_pending(
        &self,
        generation: u32,
        token: &str,
        apply: impl FnOnce(&Pending) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut pending = self.0.lock().map_err(|_| "手动记录会话不可用")?;
        let p = pending.as_ref().ok_or("请重新采集手动值")?;
        if p.created.elapsed() >= TTL {
            *pending = None;
            return Err("手动记录已过期，请重新采集".into());
        }
        if p.generation != generation || p.token != token {
            return Err("手动记录上下文已变化，请重新采集".into());
        }
        apply(p)?;
        *pending = None;
        Ok(())
    }
}
#[tauri::command]
pub(crate) async fn manual_capture(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Option<Captured>, String> {
    let service = app.state::<Service>();
    match request {
        Request::Cancel { token } => {
            service.cancel(&token)?;
            Ok(None)
        }
        Request::Capture {
            generation,
            host_id,
            source,
            selected,
            scene_id,
        } => {
            let session = app.state::<crate::previs::SharedSession>();
            session
                .lock()
                .map_err(|_| "工程会话不可用")?
                .check_snapshot(generation)?;
            let collected = app
                .state::<crate::execution::Service>()
                .capture(host_id, source, selected)
                .await?;
            // Verify again after async IO; a switched/saved/edited project cannot inherit the result.
            let current = session
                .lock()
                .map_err(|_| "工程会话不可用")?
                .check_snapshot(generation)?;
            collected.capture.check(&current)?;
            let merge = scene_id
                .as_deref()
                .map(|id| current.prepare_manual_scene_merge(&collected.capture, id))
                .transpose()?;
            service.prepare(generation, collected, merge).map(Some)
        }
    }
}

#[cfg(test)]
#[path = "manual_capture_tests.rs"]
mod tests;
