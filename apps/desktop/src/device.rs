use stagemaster_device_host::{Ble, Problem, Request, Service, Snapshot};

pub type Connections = std::sync::Arc<Service<Ble>>;

#[tauri::command]
pub async fn device_request(
    state: tauri::State<'_, Connections>,
    request: Request,
) -> Result<Snapshot, Problem> {
    state.request(request)
}
