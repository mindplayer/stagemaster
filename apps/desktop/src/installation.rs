mod destination;
mod exit;
use destination::Destination;
pub(crate) use exit::allow_exit;
use serde::{Deserialize, Serialize};
use stagemaster_device_upload::{Prepared, Snapshot};
use tauri::Manager;

pub(crate) type Service = stagemaster_device_upload::Service<
    stagemaster_device_host::Service<stagemaster_device_host::Ble>,
>;
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Status,
    Cancel { id: String },
    Resume { id: String, epoch: u32 },
    Forget { id: String },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct View {
    installation: Snapshot,
    destination: Destination,
}
fn view(app: &tauri::AppHandle) -> Result<View, String> {
    Ok(View {
        installation: app.state::<Service>().snapshot()?,
        destination: destination::read(&app.state::<crate::device::Connections>())?,
    })
}
#[tauri::command]
pub(crate) async fn installation_request(
    app: tauri::AppHandle,
    request: Request,
) -> Result<View, String> {
    let service = app.state::<Service>();
    match request {
        Request::Status => {}
        Request::Cancel { id } => {
            service.cancel(&id)?;
        }
        Request::Resume { id, epoch } => {
            service.resume(&id, epoch)?;
        }
        Request::Forget { id } => {
            service.forget(&id)?;
        }
    }
    view(&app)
}
#[tauri::command]
pub(crate) async fn installation_start(
    app: tauri::AppHandle,
    generation: u32,
    token: String,
    epoch: u32,
    device_id: String,
) -> Result<View, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let package = app.state::<crate::package::Service>();
        let prepared = Prepared::new(package.prepared(generation, &token)?)?;
        // Keep the same package -> recovery lock order as file export; prevent cache changes
        // between the final token check and creation of the immutable background task.
        let _package_operation = package.operation()?;
        let operations = app.state::<crate::recovery::Service>();
        let _operation = operations
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        // Recheck the cache and applied document after preparation; retain bytes in the task.
        package.prepared(generation, &token)?;
        app.state::<crate::previs::SharedSession>()
            .lock()
            .map_err(|_| "工程会话发生错误")?
            .export_source(generation)?;
        // spawn_blocking does not enter Tauri's separate runtime implicitly on every host.
        let service = app.state::<Service>();
        let _runtime = tauri::async_runtime::handle().inner().enter();
        service.start(prepared, epoch, &device_id)?;
        view(&app)
    })
    .await
    .map_err(|_| "安装任务未能开始，请重新读取任务状态".to_string())?
}
