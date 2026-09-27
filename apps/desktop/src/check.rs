//! Desktop envelope owns freshness and availability; the project module owns diagnostics.
use serde::Serialize;
use stagemaster_project::CheckReport;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
pub(crate) struct Service(Mutex<()>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckResponse {
    generation: u32,
    report: CheckReport,
    device_release: &'static str,
}

#[tauri::command]
pub(crate) async fn check_request(
    app: tauri::AppHandle,
    generation: u32,
) -> Result<CheckResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "另一项工程检查正在结束，请稍后重试")?;
        let document = {
            let session = app.state::<crate::previs::SharedSession>();
            session
                .lock()
                .map_err(|_| "工程会话发生错误，请重启应用")?
                .check_snapshot(generation)?
        };
        // Deliberately outside the session lock: editing and preview polling remain available.
        Ok(CheckResponse {
            generation,
            report: document.check(),
            device_release: "unavailable",
        })
    })
    .await
    .map_err(|_| "工程检查未完成，请重新检查".to_string())?
}
