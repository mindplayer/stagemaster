//! Native snapshot export; no project mutation or playback/output authority.
use serde::Serialize;
use stagemaster_project_store::PatchReportFile;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
pub(crate) struct Service(Mutex<()>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportResponse {
    generation: u32,
    path: Option<String>,
    fixture_count: usize,
    warning: Option<String>,
}

#[tauri::command]
pub(crate) async fn patch_report_export(
    app: tauri::AppHandle,
    generation: u32,
) -> Result<ExportResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "已有配灯表正在导出，请稍后重试")?;
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let (document, source) = {
            let shared = app.state::<crate::previs::SharedSession>();
            let session = shared.lock().map_err(|_| "工程会话发生错误")?;
            (
                session.check_snapshot(generation)?,
                session.export_source(generation)?,
            )
        };
        let report = document.patch_report()?;
        let mut result = ExportResponse {
            generation,
            path: None,
            fixture_count: report.fixture_count(),
            warning: None,
        };
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导出配灯表")
            .set_file_name("配灯表.csv")
            .add_filter("配灯表 CSV", &["csv"])
            .blocking_save_file()
        else {
            return Ok(result);
        };
        let path = file.into_path().map_err(|_| "请选择本机保存位置")?;
        let mut file = PatchReportFile::select(&path, source.as_deref())?;
        result.warning = file.save(&report)?;
        result.path = Some(file.path().to_string_lossy().into_owned());
        Ok(result)
    })
    .await
    .map_err(|_| "配灯表导出未完成，请重试".to_string())?
}
