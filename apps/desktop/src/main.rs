#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod check;
mod device;
mod installation;
mod lifecycle;
mod package;
mod preview;
mod previs;
mod recent;
mod recovery;
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
    OpenRecent {
        generation: u32,
        id: String,
    },
    Recover {
        generation: u32,
        id: String,
        token: String,
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
        let service = app.state::<recovery::Service>();
        let _operation = service
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let state = app.state::<previs::SharedSession>();
        let mut session = state.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
        let result = dispatch(&app, request, &mut session);
        let checkpoint = session.checkpoint();
        drop(session);
        // Disk I/O does not hold the mutex used by playback/viewport polling.
        let status = if matches!(result, Ok(true)) {
            recovery::Status::default()
        } else {
            service.status_after(checkpoint)
        };
        let mut session = state.lock().map_err(|_| "工程会话发生错误，请重启应用")?;
        session.recovery = status;
        let snapshot = session.snapshot();
        drop(session);
        if result? {
            app.state::<installation::Service>().begin_shutdown();
            app.exit(0);
        }
        Ok(snapshot)
    })
    .await
    .map_err(|_| "工程操作未完成".to_string())?
}
fn dispatch(
    app: &tauri::AppHandle,
    request: Request,
    session: &mut Session,
) -> Result<bool, String> {
    match request {
        Request::Snapshot => {}
        Request::New { generation } => session.create(app, generation)?,
        Request::Open { generation } => session.open(app, generation)?,
        Request::OpenRecent { generation, id } => session.open_recent(app, generation, &id)?,
        Request::Recover {
            generation,
            id,
            token,
        } => session.recover(app, generation, &id, &token)?,
        Request::Save {
            generation,
            save_as,
        } => {
            session.save(app, generation, save_as)?;
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
            if !installation::allow_exit(app)? {
                return Ok(false);
            }
            return session.allow_replace(app);
        }
    }
    Ok(false)
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
    let devices = Arc::new(stagemaster_device_host::Service::new(
        device::backend().expect("设备连接配置无效"),
    ));
    let installation = installation::Service::new(devices.clone());
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(Session::default())))
        .manage(previs::Bridge::default())
        .manage(check::Service::default())
        .manage(package::Service::default())
        .manage(devices)
        .manage(installation)
        .setup(|app| {
            let data = recovery::directory(app)?
                .parent()
                .ok_or("缺少数据目录")?
                .to_path_buf();
            app.manage(audio::Service::new(data.join("audio")));
            app.manage(recent::Service::new(data.join("navigation")));
            app.manage(recovery::Service::new(recovery::directory(app)?));
            Ok(())
        })
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
            audio::audio_request,
            audio::prepare::audio_prepare,
            audio::prepare::audio_cancel,
            preview_request,
            check::check_request,
            package::package_build,
            package::package_export,
            recovery::recovery_request,
            recent::recent_request,
            device::device_request,
            installation::installation_request,
            installation::installation_start,
            previs::previs_request
        ])
        .build(tauri::generate_context!())
        .expect("舞台大师桌面应用启动失败")
        .run(lifecycle::handle_run);
}
