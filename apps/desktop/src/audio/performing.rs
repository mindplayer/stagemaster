use super::Command;
use stagemaster_audio::Position;
use tauri::Manager;

pub(super) fn execute(
    app: &tauri::AppHandle,
    generation: u32,
    command: Command,
    cancellation: u64,
) -> Result<Position, String> {
    let service = app.state::<super::Service>();
    let session = app.state::<crate::previs::SharedSession>();
    let request = {
        let mut guard = session.lock().map_err(|_| "工程会话发生错误")?;
        if matches!(command, Command::Play | Command::Seek { .. }) {
            service.check_cancellation_version(cancellation)?;
        }
        if let Some(request) = guard.audio_preparation(generation, &command)? {
            request
        } else {
            return service.apply_immediate(&mut guard, generation, command);
        }
    };
    let _guard = service.begin_preparation(cancellation)?;
    let prepared = request.prepare(&service.cancelled)?;
    let mut guard = session.lock().map_err(|_| "工程会话发生错误")?;
    service.check_cancelled()?;
    service.check_cancellation_version(cancellation)?;
    guard.apply_audio_preparation(generation, prepared)
}

impl super::Service {
    pub(super) fn apply_immediate(
        &self,
        session: &mut crate::session::Session,
        generation: u32,
        command: Command,
    ) -> Result<Position, String> {
        let interrupt = matches!(command, Command::Pause | Command::Stop);
        let position = session.audio_request(generation, command)?;
        // A stale window must not cancel the new project's preparation.
        if interrupt {
            self.cancel();
        }
        Ok(position)
    }
}
