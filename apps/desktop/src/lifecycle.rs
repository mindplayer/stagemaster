use crate::{device, installation, previs};
use tauri::{Emitter, Manager};

pub(crate) fn handle_run(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if matches!(event, tauri::RunEvent::Exit) {
        if let Err(error) =
            tauri::async_runtime::block_on(app.state::<installation::Service>().shutdown())
        {
            eprintln!("安装任务退出清理：{error}");
        }
        if let Err(error) =
            tauri::async_runtime::block_on(app.state::<device::Connections>().shutdown())
        {
            eprintln!("设备连接退出清理：{error}");
        }
        tauri::async_runtime::block_on(app.state::<previs::Bridge>().close());
    }
    if let tauri::RunEvent::ExitRequested {
        api, code: None, ..
    } = event
    {
        api.prevent_exit();
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.emit("project-close-requested", ());
        }
    }
}
