use stagemaster_audio::{LoopRange, Position};
use tauri::Manager;
pub(super) fn configure(
    app: &tauri::AppHandle,
    generation: u32,
    range: Option<LoopRange>,
    cancellation: u64,
) -> Result<Position, String> {
    let service = app.state::<super::Service>();
    let _guard = service.begin_preparation(cancellation)?;
    let session = app.state::<crate::previs::SharedSession>();
    let request = {
        let guard = session.lock().map_err(|_| "工程会话发生错误")?;
        guard.audio_loop_request(generation, range)?
    };
    let prepared = request.prepare()?;
    let mut guard = session.lock().map_err(|_| "工程会话发生错误")?;
    service.check_cancelled()?;
    service.check_cancellation_version(cancellation)?;
    guard.apply_audio_loop(generation, prepared)
}
