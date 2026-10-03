use super::{Runner, job::Failure};
use serde::Serialize;
use stagemaster_audio::PerformanceObservation;
use stagemaster_live_host::media::{ControlRequest, MediaCommand};
use stagemaster_runtime::Code;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LoopState {
    pub region: usize,
    pub name: String,
    pub pass: String,
    pub exit_requested: bool,
    pub pending_exit: Option<bool>,
}
impl Runner {
    pub(super) fn loop_state(&self, native: PerformanceObservation) -> Option<LoopState> {
        let position = native.snapshot.position;
        let index = position.region?;
        let track = self.doc.audio_timeline()?;
        let region = track.loop_regions.iter().filter(|r| r.enabled).nth(index)?;
        Some(LoopState {
            region: index,
            name: region.name.clone(),
            pass: position.pass?.to_string(),
            exit_requested: position.exit_requested,
            pending_exit: native.snapshot.pending_exit.map(|intent| intent.requested),
        })
    }
    pub(super) fn exit_loop(&mut self, request: ControlRequest) -> Result<(), Failure> {
        let MediaCommand::ExitLoop {
            instance,
            region,
            pass,
            requested,
        } = request.command
        else {
            return Err(Failure::Rejected(Code::State));
        };
        let before = self.wait(request, |r| r.native())?;
        if let Some(problem) = before.snapshot.problem {
            return Err(Failure::Problem(problem.into()));
        }
        let sequence = before.snapshot.render.map_or(0, |r| r.sequence);
        self.transport
            .exit_performance(&instance.to_string(), region, pass, requested)
            .map_err(|_| Failure::Rejected(Code::Selection))?;
        let confirmed = self.wait(request, |r| {
            Ok(r.native()?.filter(|n| {
                n.instance == instance
                    && n.snapshot.pending_exit.is_none()
                    && n.snapshot.render.is_some_and(|v| v.sequence > sequence)
            }))
        })?;
        if let Some(problem) = confirmed.snapshot.problem {
            return Err(Failure::Problem(problem.into()));
        }
        if confirmed.snapshot.control_problem.is_some() {
            return Err(Failure::Rejected(Code::Selection));
        }
        Ok(())
    }
}
