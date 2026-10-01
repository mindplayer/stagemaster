//! Native mode-file dialogs are independent of authoring and playback authority.
use serde::Serialize;
use stagemaster_project::{ProfileDefinition, ProfileSource};
use stagemaster_project_store::ProfileFileStore;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
pub(crate) struct Service(Mutex<()>);
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportResponse {
    generation: u32,
    file_name: String,
    source: ProfileSource,
    definition: ProfileDefinition,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportResponse {
    generation: u32,
    profile_id: String,
    revision: String,
    path: Option<String>,
    warning: Option<String>,
}

#[tauri::command]
pub(crate) async fn profile_file_import(
    app: tauri::AppHandle,
    generation: u32,
) -> Result<Option<ImportResponse>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "另一项模式文件操作尚未结束")?;
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        {
            let shared = app.state::<crate::previs::SharedSession>();
            let session = shared.lock().map_err(|_| "工程会话发生错误")?;
            session.export_source(generation)?;
        }
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导入灯具模式")
            .add_filter("StageMaster 灯具模式", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "请选择本机灯具模式文件")?;
        let file = ProfileFileStore::read(&path)?;
        Ok(Some(ImportResponse {
            generation,
            file_name: path
                .file_name()
                .ok_or("模式文件名无效")?
                .to_string_lossy()
                .into_owned(),
            source: file.source().clone(),
            definition: file.definition().clone(),
        }))
    })
    .await
    .map_err(|_| "读取灯具模式未完成，请重试".to_string())?
}

#[tauri::command]
pub(crate) async fn profile_file_export(
    app: tauri::AppHandle,
    generation: u32,
    profile_id: String,
) -> Result<ExportResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "另一项模式文件操作尚未结束")?;
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let document = app
            .state::<crate::previs::SharedSession>()
            .lock()
            .map_err(|_| "工程会话发生错误")?
            .check_snapshot(generation)?;
        let profile = document.profile_file(&profile_id)?;
        let mut response = ExportResponse {
            generation,
            profile_id,
            revision: profile.source().revision.clone(),
            path: None,
            warning: None,
        };
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导出灯具模式")
            .set_file_name("灯具模式.smfixture.json")
            .add_filter("StageMaster 灯具模式", &["json"])
            .blocking_save_file()
        else {
            return Ok(response);
        };
        let path = file.into_path().map_err(|_| "请选择本机保存位置")?;
        let mut destination = ProfileFileStore::select(&path)?;
        response.warning = destination.save(&profile)?;
        response.path = Some(destination.path().to_string_lossy().into_owned());
        Ok(response)
    })
    .await
    .map_err(|_| "导出灯具模式未完成，请重试".to_string())?
}
