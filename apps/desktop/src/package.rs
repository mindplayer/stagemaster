//! Snapshot compilation and user-selected export; no installation or physical output authority.
use serde::Serialize;
use stagemaster_project::{PackageIssue, PackageReport, PackageSelection};
use stagemaster_project_store::PackageFile;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(crate) struct Service {
    gate: Mutex<()>,
    prepared: Mutex<Option<Prepared>>,
}
struct Prepared {
    generation: u32,
    token: String,
    bytes: Arc<[u8]>,
}
impl Service {
    // All cache mutations and consumers acquire this before recovery.operations.
    pub(crate) fn operation(&self) -> Result<MutexGuard<'_, ()>, String> {
        self.gate
            .try_lock()
            .map_err(|_| "另一项播放包操作尚未结束，请稍后重试".into())
    }
    pub(crate) fn prepared(&self, generation: u32, token: &str) -> Result<Arc<[u8]>, String> {
        let prepared = self.prepared.lock().map_err(|_| "播放包会话发生错误")?;
        let prepared = prepared.as_ref().ok_or("请先生成播放包")?;
        if prepared.generation != generation || prepared.token != token {
            return Err("播放包结果已失效，请重新生成".into());
        }
        Ok(prepared.bytes.clone())
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildResponse {
    generation: u32,
    token: Option<String>,
    report: Option<PackageReport>,
    issues: Vec<PackageIssue>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportResponse {
    path: Option<String>,
    warning: Option<String>,
}

#[tauri::command]
pub(crate) async fn package_build(
    app: tauri::AppHandle,
    generation: u32,
    selection: Vec<PackageSelection>,
) -> Result<BuildResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service.operation()?;
        *service.prepared.lock().map_err(|_| "播放包会话发生错误")? = None;
        let document = {
            let shared = app.state::<crate::previs::SharedSession>();
            shared
                .lock()
                .map_err(|_| "工程会话发生错误")?
                .check_snapshot(generation)?
        };
        match document.build_package(&selection) {
            Ok(built) => {
                let token = uuid::Uuid::new_v4().to_string();
                *service.prepared.lock().map_err(|_| "播放包会话发生错误")? = Some(Prepared {
                    generation,
                    token: token.clone(),
                    bytes: built.bytes.into(),
                });
                Ok(BuildResponse {
                    generation,
                    token: Some(token),
                    report: Some(built.report),
                    issues: vec![],
                })
            }
            Err(issues) => Ok(BuildResponse {
                generation,
                token: None,
                report: None,
                issues,
            }),
        }
    })
    .await
    .map_err(|_| "播放包生成未完成，请重试".to_string())?
}
#[tauri::command]
pub(crate) async fn package_export(
    app: tauri::AppHandle,
    generation: u32,
    token: String,
) -> Result<ExportResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service.operation()?;
        let prepared = service.prepared.lock().map_err(|_| "播放包会话发生错误")?;
        let prepared = prepared.as_ref().ok_or("请先生成播放包")?;
        if prepared.generation != generation || prepared.token != token {
            return Err("播放包结果已失效，请重新生成".into());
        }
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let source = {
            let session = app.state::<crate::previs::SharedSession>();
            session
                .lock()
                .map_err(|_| "工程会话发生错误")?
                .export_source(generation)?
        };
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导出播放包")
            .set_file_name("节目.smpkg")
            .add_filter("舞台大师播放包", &["smpkg"])
            .blocking_save_file()
        else {
            return Ok(ExportResponse {
                path: None,
                warning: None,
            });
        };
        let path = file.into_path().map_err(|_| "请选择本机保存位置")?;
        let mut target = PackageFile::select(&path, source.as_deref())?;
        let warning = target.save(&prepared.bytes)?;
        Ok(ExportResponse {
            path: Some(target.path().to_string_lossy().into_owned()),
            warning,
        })
    })
    .await
    .map_err(|_| "播放包导出未完成，请重试".to_string())?
}
