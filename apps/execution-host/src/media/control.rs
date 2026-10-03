use super::{Runner, job::Failure};
use stagemaster_live_host::media::{ControlRequest, MediaCommand};
use stagemaster_runtime::Code;

impl Runner {
    pub(super) fn complete_request(
        &mut self,
        request: ControlRequest,
        result: Result<(), Code>,
    ) -> Result<(), Failure> {
        self.wait(request, |r| {
            match r.port.complete_control(request.ticket, result) {
                Ok(()) => Ok(Some(())),
                Err(Code::Busy) => Ok(None),
                // Recheck the current ticket in guard: a replaced/expired loop request
                // cannot turn a healthy running source into a playback failure.
                Err(Code::State) if matches!(request.command, MediaCommand::ExitLoop { .. }) => {
                    Ok(None)
                }
                Err(e) => Err(e.to_string()),
            }
        })
    }

    pub(super) fn observe_control(
        &mut self,
        media: stagemaster_live_host::media::MediaState,
    ) -> Result<bool, String> {
        if let Some(control) = media.control
            && self
                .request
                .is_some_and(|r| r.ticket == control.request.ticket)
            && let Some(result) = control.result
        {
            if result.is_err() && !matches!(control.request.command, MediaCommand::ExitLoop { .. })
            {
                return Err("后台未确认当前音乐操作，音源已停止".into());
            }
            if self.is_end_seek(control.request.command) {
                self.active = None;
                self.pending_sample = None;
            }
            self.request = None;
        }
        if let Some(control) = media.control.filter(|c| c.result.is_none())
            && self.seen != Some(control.request.ticket)
        {
            self.seen = Some(control.request.ticket);
            self.request = Some(control.request);
            if let Ok(mut view) = self.view.lock() {
                view.problem = None;
                view.status = "preparing";
            }
            match self.execute(control.request, media.group.key) {
                Ok(()) => {}
                Err(Failure::Superseded) => {
                    if matches!(control.request.command, MediaCommand::Recover { .. }) {
                        self.fail("音乐重新准备已取消，请重试".into());
                    } else if !matches!(control.request.command, MediaCommand::ExitLoop { .. }) {
                        self.cancel_job();
                    }
                }
                Err(Failure::Rejected(code)) => {
                    let completion = self.complete_request(control.request, Err(code));
                    match completion {
                        Ok(()) | Err(Failure::Superseded) => {}
                        Err(Failure::Problem(problem)) => return Err(problem),
                        Err(Failure::Rejected(code)) => return Err(code.to_string()),
                    }
                }
                Err(Failure::Problem(problem)) => return Err(problem),
            }
            if matches!(control.request.command, MediaCommand::ExitLoop { .. })
                && let Ok(mut view) = self.view.lock()
            {
                view.status = if self.transport.position().playing {
                    "playing"
                } else {
                    "paused"
                };
            }
            return Ok(true);
        }
        // Explicit EOF is completed through its control receipt, not a second termination request.
        Ok(media.control.is_some_and(|control| {
            control.result.is_none()
                && self.request == Some(control.request)
                && self.is_end_seek(control.request.command)
        }))
    }
}
