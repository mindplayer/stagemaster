use stagemaster_device_host::{Ble, Problem, Request, Service, Snapshot};

pub type Connections = std::sync::Arc<Service<Ble>>;

// Both builds retain the same fallible startup contract; only the explicit
// development feature currently loads a credential provider that can fail.
#[cfg_attr(
    not(feature = "development-device-access"),
    allow(clippy::unnecessary_wraps)
)]
pub fn backend() -> Result<Ble, Problem> {
    #[cfg(feature = "development-device-access")]
    if let Some(path) = std::env::var_os("STAGEMASTER_CONTROLLER_CONFIGURATION") {
        let config =
            stagemaster_device_host::read_development_configuration(std::path::Path::new(&path))?;
        return Ok(Ble::with_development_configuration(config));
    }
    Ok(Ble::default())
}

#[tauri::command]
pub async fn device_request(
    state: tauri::State<'_, Connections>,
    request: Request,
) -> Result<Snapshot, Problem> {
    state.request(request)
}
