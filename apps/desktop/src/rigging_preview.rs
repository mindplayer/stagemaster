//! Read-only draft projection through the same project edit used by Apply.
use serde::{Deserialize, Serialize};
use stagemaster_project::{Document, EditCommand, FixturePlacement, StageEdit};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
pub(crate) struct Service(Mutex<()>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Request {
    generation: u32,
    command: StageEdit,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Projection {
    generation: u32,
    placements: Vec<FixturePlacement>,
    changed: bool,
}

pub(crate) fn project(mut document: Document, request: Request) -> Result<Projection, String> {
    let StageEdit::AttachFixtures {
        ref fixture_ids, ..
    } = request.command
    else {
        return Err("此预览只接受灯具挂接操作".into());
    };
    let ids = fixture_ids.clone();
    let before = document.view().stage;
    document.edit(EditCommand::Stage {
        command: request.command,
    })?;
    let after = document.view().stage;
    // Attachment array order is not a spatial change. Avoid spurious history for Keep Position.
    let changed = ids.iter().any(|id| {
        let old = before.placements.iter().find(|p| p.fixture_id == *id);
        let new = after.placements.iter().find(|p| p.fixture_id == *id);
        serde_json::to_value(old).ok() != serde_json::to_value(new).ok()
            || before
                .attachments
                .iter()
                .find(|a| a.fixture_id == *id)
                .map(|a| &a.construction_id)
                != after
                    .attachments
                    .iter()
                    .find(|a| a.fixture_id == *id)
                    .map(|a| &a.construction_id)
    });
    Ok(Projection {
        generation: request.generation,
        placements: after
            .placements
            .into_iter()
            .filter(|p| ids.contains(&p.fixture_id))
            .collect(),
        changed,
    })
}

#[tauri::command]
pub(crate) async fn rigging_preview(
    app: tauri::AppHandle,
    request: Request,
) -> Result<Projection, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<Service>();
        let _permit = service
            .0
            .try_lock()
            .map_err(|_| "挂接预览正在处理，请重试")?;
        let state = app.state::<crate::previs::SharedSession>();
        let document = state
            .lock()
            .map_err(|_| "工程会话发生错误，请重启应用")?
            .check_snapshot(request.generation)?;
        // Validation and projection do not hold the playback/session mutex.
        project(document, request)
    })
    .await
    .map_err(|_| "挂接预览未完成，请重试".to_string())?
}
