use crate::{device, installation, previs};
use tauri::{Emitter, Manager};

pub(crate) fn handle_run(app: &tauri::AppHandle, event: tauri::RunEvent) {
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

/// Keep the event loop alive while adapters finish work and IPC replies drain.
// Never call block_on from RunEvent::Exit: WebKit may need that same thread.
pub(crate) async fn shutdown(app: &tauri::AppHandle) {
    app.state::<previs::Bridge>().begin_shutdown();
    if let Err(error) = app.state::<installation::Service>().shutdown().await {
        eprintln!("安装任务退出清理：{error}");
    }
    if let Err(error) = app.state::<device::Connections>().shutdown().await {
        eprintln!("设备连接退出清理：{error}");
    }
    app.state::<previs::Bridge>().close().await;
}
