use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

pub(crate) fn allow_exit(app: &tauri::AppHandle) -> Result<bool, String> {
    let snapshot = app.state::<super::Service>().snapshot()?;
    if snapshot.task.is_none_or(|task| task.phase.terminal()) {
        return Ok(true);
    }
    let message = concat!(
        "节目安装尚未核实完成。退出将停止本机通信，不会撤销设备中已经发生的安装。",
        "任务不会跨应用重启自动恢复；请保留原播放包，之后重新连接原设备核对。",
    );
    let result = app
        .dialog()
        .message(message)
        .parent(&app.get_webview_window("main").ok_or("主窗口已关闭")?)
        .title("结束安装通信并退出？")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "退出应用".into(),
            "继续处理".into(),
        ))
        .blocking_show_with_result();
    Ok(matches!(result, MessageDialogResult::Ok)
        || matches!(result,MessageDialogResult::Custom(label) if label=="退出应用"))
}
