use stagemaster_audio::{LoopRange, Position};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
struct PreparationGuard<'a>(&'a AtomicBool);
impl Drop for PreparationGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
pub(super) fn configure(
    app: &tauri::AppHandle,
    generation: u32,
    range: Option<LoopRange>,
) -> Result<Position, String> {
    let service = app.state::<super::Service>();
    service
        .loop_preparing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| "正在准备另一个循环，请稍后")?;
    let _guard = PreparationGuard(&service.loop_preparing);
    let session = app.state::<crate::previs::SharedSession>();
    let request = {
        let guard = session.lock().map_err(|_| "工程会话发生错误")?;
        guard.audio_loop_request(generation, range)?
    };
    let prepared = request.prepare()?;
    let mut guard = session.lock().map_err(|_| "工程会话发生错误")?;
    guard.apply_audio_loop(generation, prepared)
}
