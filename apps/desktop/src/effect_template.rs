//! Local template IO and a single, expiring review. No device or playback authority.
pub(crate) mod files;
mod pending;
pub(crate) use pending::Service;
use tauri::Manager;

#[tauri::command]
pub(crate) fn effect_template_cancel(app: tauri::AppHandle, token: &str) -> Result<(), String> {
    let result = app.state::<Service>().cancel(token);
    drop(app);
    result
}
