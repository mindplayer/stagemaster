//! Native snapshot export; no project mutation or playback/output authority.
use serde::Serialize;
use stagemaster_project_store::SequenceReportFile;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportResponse {
    generation: u32,
    path: Option<String>,
    sequence_id: String,
    sequence_name: String,
    step_count: usize,
    warning: Option<String>,
}

#[tauri::command]
pub(crate) async fn sequence_report_export(
    app: tauri::AppHandle,
    generation: u32,
    sequence_id: String,
) -> Result<ExportResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<crate::report_export::Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "已有交接资料正在导出，请稍后重试")?;
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
        let report = document.sequence_report(&sequence_id)?;
        let mut result = ExportResponse {
            generation,
            path: None,
            sequence_id: report.sequence_id().into(),
            sequence_name: report.sequence_name().into(),
            step_count: report.step_count(),
            warning: None,
        };
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导出节目单")
            .set_file_name("节目单.csv")
            .add_filter("节目单 CSV", &["csv"])
            .blocking_save_file()
        else {
            return Ok(result);
        };
        let path = file.into_path().map_err(|_| "请选择本机保存位置")?;
        let mut file = SequenceReportFile::select(&path, source.as_deref())?;
        result.warning = file.save(&report)?;
        result.path = Some(file.path().to_string_lossy().into_owned());
        Ok(result)
    })
    .await
    .map_err(|_| "节目单导出未完成，请重试".to_string())?
}
