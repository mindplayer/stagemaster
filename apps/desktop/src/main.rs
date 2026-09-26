#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod preview;
mod previs;
mod session;
use serde::Deserialize;
use session::{Session, Snapshot};
use stagemaster_project::{EditCommand, FixturePlacement};
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Request {
    Snapshot,
    New {
        generation: u32,
    },
    Open {
        generation: u32,
    },
    Save {
        generation: u32,
        #[serde(rename = "saveAs")]
        save_as: bool,
    },
    Edit {
        generation: u32,
        command: EditCommand,
    },
    PrevisPlacement {
        generation: u32,
        version: String,
        placement: FixturePlacement,
    },
    History {
        generation: u32,
        redo: bool,
    },
    Close,
}
#[tauri::command]
async fn project_request(app: tauri::AppHandle, request: Request) -> Result<Snapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<previs::SharedSession>();
        let mut session = state.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
        match request {
            Request::Snapshot => {}
            Request::New { generation } => session.create(&app, generation)?,
            Request::Open { generation } => session.open(&app, generation)?,
            Request::Save {
                generation,
                save_as,
            } => {
                session.save(&app, generation, save_as)?;
            }
            Request::Edit {
                generation,
                command,
            } => session.edit(generation, command)?,
            Request::PrevisPlacement {
                generation,
                version,
                placement,
            } => {
                session.place_from_viewport(generation, &version, placement)?;
            }
            Request::History { generation, redo } => session.history(generation, redo)?,
            Request::Close => {
                if session.allow_replace(&app)? {
                    app.exit(0);
                }
            }
        }
        Ok(session.snapshot())
    })
    .await
    .map_err(|_| "工程操作未完成".to_string())?
}
#[tauri::command]
async fn preview_request(
    app: tauri::AppHandle,
    request: preview::Request,
) -> Result<preview::Snapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<previs::SharedSession>();
        let mut session = state.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
        session.preview(request)
    })
    .await
    .map_err(|_| "预览操作未完成".to_string())?
}
fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(Session::default())))
        .manage(previs::Bridge::default())
        .plugin(tauri_plugin_dialog::init())
        .menu(|app| {
            let app_menu = Submenu::with_items(
                app,
                "舞台大师",
                true,
                &[
                    &PredefinedMenuItem::about(app, Some("关于舞台大师"), None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, Some("隐藏舞台大师"))?,
                    &MenuItem::with_id(app, "quit", "退出舞台大师", true, Some("CmdOrCtrl+Q"))?,
                ],
            )?;
            let edit = Submenu::with_items(
                app,
                "编辑",
                true,
                &[
                    &PredefinedMenuItem::cut(app, Some("剪切"))?,
                    &PredefinedMenuItem::copy(app, Some("复制"))?,
                    &PredefinedMenuItem::paste(app, Some("粘贴"))?,
                    &PredefinedMenuItem::select_all(app, Some("全选"))?,
                ],
            )?;
            let window = Submenu::with_items(
                app,
                "窗口",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, Some("最小化"))?,
                    &PredefinedMenuItem::close_window(app, Some("关闭窗口"))?,
                ],
            )?;
            Menu::with_items(app, &[&app_menu, &edit, &window])
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit"
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.close();
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("project-close-requested", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            project_request,
            preview_request,
            previs::previs_request
        ])
        .build(tauri::generate_context!())
        .expect("舞台大师桌面应用启动失败")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
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
        });
}
