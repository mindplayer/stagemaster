use super::{Runner, job::Failure};
use stagemaster_live_host::media::{ControlRequest, MediaCommand};

impl Runner {
    pub(super) fn is_end_seek(&self, command: MediaCommand) -> bool {
        matches!(command, MediaCommand::Seek { position_ms, .. }
            if self.doc.audio_timeline().is_some_and(|t| position_ms == t.duration_ms()))
    }

    pub(super) fn seek_end(&mut self, request: ControlRequest) -> Result<(), Failure> {
        let MediaCommand::Seek { position_ms, .. } = request.command else {
            return Err("当前操作不是末尾定位".to_string().into());
        };
        self.transport.pause();
        let source = self
            .transport
            .seek_preparation(position_ms, false)?
            .ok_or_else(|| "后台正式音源未准备".to_string())?
            .prepare(&self.cancel)?;
        self.guard(request)?;
        self.transport.apply_seek(source)?;
        // A real prepared EOF source is authoritative without a synthetic render callback.
        self.wait(request, |runner| {
            let Some(native) = runner.native()? else {
                return Ok(None);
            };
            if let Some(problem) = native.snapshot.problem {
                return Err(problem.into());
            }
            if native.snapshot.stopped
                || native.requested_playing
                || !native.snapshot.position.ended
                || runner.transport.position().position_ms != position_ms
            {
                return Err("音源尚未确认末尾位置".into());
            }
            Ok(Some(()))
        })?;
        self.pending_sample = None;
        // Retain the active key for failure cleanup until Host confirms the original request.
        Ok(())
    }
}
