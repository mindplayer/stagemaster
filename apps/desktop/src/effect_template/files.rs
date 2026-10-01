use super::{Service, pending::Imported};
use serde::Serialize;
use stagemaster_project_store::EffectTemplateFileStore;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Exported {
    generation: u32,
    effect_id: String,
    path: Option<String>,
    warning: Option<String>,
}
#[tauri::command]
pub(crate) async fn effect_template_import(
    app: tauri::AppHandle,
    generation: u32,
    scene_id: String,
    fixture_ids: Vec<String>,
) -> Result<Option<Imported>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .gate
            .try_lock()
            .map_err(|_| "另一项灯效模板文件操作尚未结束")?;
        service.clear()?;
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let document = app
            .state::<crate::previs::SharedSession>()
            .lock()
            .map_err(|_| "工程会话发生错误")?
            .check_snapshot(generation)?;
        let Some(file) = app
            .dialog()
            .file()
            .set_title("导入灯效模板")
            .add_filter("StageMaster 灯效模板", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "请选择本机灯效模板文件")?;
        let file = EffectTemplateFileStore::read(&path)?;
        let review = document.review_effect_template(&file, &scene_id, &fixture_ids)?;
        let name = path
            .file_name()
            .ok_or("模板文件名无效")?
            .to_string_lossy()
            .into_owned();
        service.prepare(generation, name, review).map(Some)
    })
    .await
    .map_err(|_| "读取灯效模板未完成，请重试".to_string())?
}
#[tauri::command]
pub(crate) async fn effect_template_export(
    app: tauri::AppHandle,
    generation: u32,
    scene_id: String,
    effect_id: String,
) -> Result<Exported, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .gate
            .try_lock()
            .map_err(|_| "另一项灯效模板文件操作尚未结束")?;
        let recovery = app.state::<crate::recovery::Service>();
        let _operation = recovery
            .operations
            .lock()
            .map_err(|_| "工程操作队列发生错误")?;
        let document = app
            .state::<crate::previs::SharedSession>()
            .lock()
            .map_err(|_| "工程会话发生错误")?
            .check_snapshot(generation)?;
        let file = document.effect_template_file(&scene_id, &effect_id)?;
        let mut response = Exported {
            generation,
            effect_id,
            path: None,
            warning: None,
        };
        let Some(destination) = app
            .dialog()
            .file()
            .set_title("导出灯效模板")
            .set_file_name("灯效.smeffect.json")
            .add_filter("StageMaster 灯效模板", &["json"])
            .blocking_save_file()
        else {
            return Ok(response);
        };
        let path = destination.into_path().map_err(|_| "请选择本机保存位置")?;
        let mut target = EffectTemplateFileStore::select(&path)?;
        response.warning = target.save(&file)?;
        response.path = Some(target.path().to_string_lossy().into_owned());
        Ok(response)
    })
    .await
    .map_err(|_| "导出灯效模板未完成，请重试".to_string())?
}
