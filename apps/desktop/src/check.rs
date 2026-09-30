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
    audio_resource: Option<AudioResourceCheck>,
    device_release: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioResourceCheck {
    file_name: String,
    resources: stagemaster_audio::ResourceHealth,
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
        let (document, source) = {
            let session = app.state::<crate::previs::SharedSession>();
            let session = session.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
            (
                session.check_snapshot(generation)?,
                session.export_source(generation)?,
            )
        };
        // Deliberately outside the session lock: editing and preview polling remain available.
        let audio_resource = document
            .audio_timeline()
            .map(|track| {
                let resources = app.state::<crate::audio::Service>().resources.inspect(
                    &track.asset.digest,
                    &track.asset.extension,
                    source.as_deref(),
                    &std::sync::atomic::AtomicBool::new(false),
                )?;
                Ok::<_, String>(AudioResourceCheck {
                    file_name: track.asset.file_name,
                    resources,
                })
            })
            .transpose()?;
        Ok(CheckResponse {
            generation,
            audio_resource,
            report: document.check(),
            device_release: "unavailable",
        })
    })
    .await
    .map_err(|_| "工程检查未完成，请重新检查".to_string())?
}
