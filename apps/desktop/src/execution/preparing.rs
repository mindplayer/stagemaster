use super::{
    manager::{Manager, Status},
    media::{self, AudioInput},
};
use crate::previs::SharedSession;
use stagemaster_execution_client::{AudioOutput, Selection};
use tauri::Manager as _;
use tokio::sync::OwnedMutexGuard;

/// The blocking task owns the reservation even if its UI future disappears during preparation.
pub(super) async fn run(
    app: tauri::AppHandle,
    mut manager: OwnedMutexGuard<Manager>,
    generation: u32,
    selection: Vec<Selection>,
    output: Option<AudioOutput>,
) -> Result<Status, String> {
    let runtime = tokio::runtime::Handle::current();
    tauri::async_runtime::spawn_blocking(move || {
        let session = app.state::<SharedSession>();
        let (document, project) = {
            let guard = session.lock().map_err(|_| "工程会话不可用")?;
            (
                guard.check_snapshot(generation)?,
                guard.export_source(generation)?,
            )
        };
        media::validate(&document, &selection, output)?;
        let audio = if let Some(output) = output {
            let track = document.audio_timeline().ok_or("工程没有音乐")?;
            let service = app.state::<crate::audio::Service>();
            let source = service.resources.resolve(
                &track.asset.digest,
                &track.asset.extension,
                project.as_deref(),
            )?;
            let mut guard = session.lock().map_err(|_| "工程会话不可用")?;
            service.stop_for_background(&mut guard, generation)?;
            Some(AudioInput { source, output })
        } else {
            None
        };
        runtime.block_on(manager.prepare(document, selection, audio))
    })
    .await
    .map_err(|_| "后台节目准备任务异常结束，请核对原后台")?
}
