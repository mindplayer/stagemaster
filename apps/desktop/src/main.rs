#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod check;
mod device;
mod effect_template;
mod execution;
mod installation;
mod lifecycle;
mod manual_capture;
mod output_control;
mod package;
mod patch_report;
mod preview;
mod previs;
mod profile_file;
mod recent;
mod recovery;
mod report_export;
mod rigging_preview;
mod sequence_report;
mod session;
mod startup;
use serde::Deserialize;
use session::{Session, Snapshot};
use stagemaster_project::{EditCommand, FixturePlacement, SpatialVector3};
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
    ApplyEffectTemplate {
        generation: u32,
        token: String,
    },
    MergeManualScene {
        generation: u32,
        token: String,
    },
    RecordManualScene {
        generation: u32,
        token: String,
        name: String,
    },
    PrevisTransform {
        generation: u32,
        version: String,
        #[serde(rename = "fixtureIds")]
        fixture_ids: Vec<String>,
        #[serde(rename = "yawDegrees")]
        yaw_degrees: String,
        #[serde(rename = "spacingScale")]
        spacing_scale: String,
    },
    PrevisObjectTranslation {
        generation: u32,
        version: String,
        targets: Vec<session::ViewportTarget>,
        #[serde(rename = "deltaMeters")]
        delta_meters: SpatialVector3,
    },
    PrevisTranslation {
        generation: u32,
        version: String,
        #[serde(rename = "fixtureIds")]
        fixture_ids: Vec<String>,
        #[serde(rename = "deltaMeters")]
        delta_meters: SpatialVector3,
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
    let cleanup_app = app.clone();
    let (snapshot, close) = tauri::async_runtime::spawn_blocking(move || {
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
        Ok::<_, String>((snapshot, result?))
    })
    .await
    .map_err(|_| "工程操作未完成".to_string())??;
    if close {
        lifecycle::shutdown(&cleanup_app).await;
        cleanup_app.exit(0);
    }
    Ok(snapshot)
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
        Request::ApplyEffectTemplate { generation, token } => {
            let review = app
                .state::<effect_template::Service>()
                .take(generation, &token)?;
            session.apply_effect_template(generation, review)?;
        }
        Request::MergeManualScene { generation, token } => {
            app.state::<manual_capture::Service>()
                .apply_merge(generation, &token, |merge| {
                    session.merge_manual_scene(generation, merge)
                })?;
        }
        Request::RecordManualScene {
            generation,
            token,
            name,
        } => {
            app.state::<manual_capture::Service>()
                .apply(generation, &token, |capture| {
                    session.record_manual_scene(generation, capture, &name)
                })?;
        }
        Request::PrevisTransform {
            generation,
            version,
            fixture_ids,
            yaw_degrees,
            spacing_scale,
        } => {
            session.transform_from_viewport(
                generation,
                &version,
                fixture_ids,
                yaw_degrees,
                spacing_scale,
            )?;
        }
        Request::PrevisObjectTranslation {
            generation,
            version,
            targets,
            delta_meters,
        } => {
            session.translate_objects_from_viewport(generation, &version, targets, delta_meters)?;
        }
        Request::PrevisTranslation {
            generation,
            version,
            fixture_ids,
            delta_meters,
        } => {
            session.translate_from_viewport(generation, &version, fixture_ids, delta_meters)?;
        }
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
        if matches!(
            request,
            preview::Request::BeginEffectDraft { .. } | preview::Request::UpdateEffectDraft { .. }
        ) {
            let preparation = state
                .lock()
                .map_err(|_| "工程会话发生错误，请重启应用")?
                .prepare_effect_draft(request)?;
            let prepared = preparation.compile()?;
            state
                .lock()
                .map_err(|_| "工程会话发生错误，请重启应用")?
                .finish_effect_draft(prepared)
        } else {
            state
                .lock()
                .map_err(|_| "工程会话发生错误，请重启应用")?
                .preview(request)
        }
    })
    .await
    .map_err(|_| "预览操作未完成".to_string())?
}
fn main() {
    let backend = device::backend().expect("设备连接配置无效");
    let runtime_access = backend.runtime_access();
    let devices = Arc::new(stagemaster_device_host::Service::new(backend));
    let installation = installation::Service::new(devices.clone());
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(Session::default())))
        .manage(previs::Bridge::default())
        .manage(check::Service::default())
        .manage(package::Service::default())
        .manage(report_export::Service::default())
        .manage(rigging_preview::Service::default())
        .manage(profile_file::Service::default())
        .manage(effect_template::Service::default())
        .manage(manual_capture::Service::default())
        .manage(devices)
        .manage(runtime_access)
        .manage(installation)
        .setup(startup::setup)
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
            manual_capture::manual_capture,
            rigging_preview::rigging_preview,
            execution::execution_request,
            audio::audio_request,
            audio::prepare::audio_prepare,
            audio::prepare::audio_cancel,
            preview_request,
            output_control::output_request,
            check::check_request,
            package::package_build,
            package::package_export,
            patch_report::patch_report_export,
            sequence_report::sequence_report_export,
            profile_file::profile_file_export,
            profile_file::profile_file_import,
            effect_template::files::effect_template_import,
            effect_template::files::effect_template_export,
            effect_template::effect_template_cancel,
            recovery::recovery_request,
            recent::recent_request,
            device::device_request,
            device::device_runtime_request,
            installation::installation_request,
            installation::installation_start,
            previs::previs_request
        ])
        .build(tauri::generate_context!())
        .expect("舞台大师桌面应用启动失败")
        .run(lifecycle::handle_run);
}
