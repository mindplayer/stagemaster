use super::{ControlState, ControlTicket, MediaCommand};
use crate::media::{Activation, MediaPort};
use stagemaster_live::media::{Prepared, Sample};
use stagemaster_runtime::Code;
use stagemaster_time::Mapping;
use std::sync::atomic::Ordering;

impl MediaPort {
    /// Observe the latest authorized intent and its outcome. Reads do not renew its deadline.
    /// # Errors
    /// Reject unregistered/dead providers and contention.
    pub fn control_state(&self) -> Result<Option<ControlState>, Code> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(Code::State);
        }
        Ok(self
            .control
            .as_ref()
            .ok_or(Code::State)?
            .try_lock()
            .map_err(|_| Code::Busy)?
            .state)
    }

    /// Stage a replacement for the current authorized start/seek. Activation runs on the original worker.
    /// # Errors
    /// Reject stale requests, invalid seek positions, missing control, contention or an occupied slot.
    pub fn stage_requested(
        &self,
        request: ControlTicket,
        prepared: Prepared,
        sample: Sample,
        mapping: Mapping,
    ) -> Result<Activation, Code> {
        let control = self
            .control
            .as_ref()
            .ok_or(Code::State)?
            .try_lock()
            .map_err(|_| Code::Busy)?;
        let intent = control.pending(request)?;
        match intent.command {
            MediaCommand::Play => {}
            MediaCommand::Seek { position_ms, .. } if sample.matches_seek(position_ms) => {}
            _ => return Err(Code::State),
        }
        if control.activated || prepared.key() != request.group {
            return Err(Code::State);
        }
        drop(control);
        self.stage_inner(prepared, sample, mapping, Some(request))
    }

    /// Report actual provider completion, never merely queue admission. A later worker tick publishes it.
    /// # Errors
    /// Reject replaced/finished tickets, duplicate completions, missing control, shutdown or contention.
    pub fn complete_control(
        &self,
        request: ControlTicket,
        result: Result<(), Code>,
    ) -> Result<(), Code> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(Code::State);
        }
        let mut control = self
            .control
            .as_ref()
            .ok_or(Code::State)?
            .try_lock()
            .map_err(|_| Code::Busy)?;
        control.pending(request)?;
        if control.completion.is_some() {
            return Err(Code::Busy);
        }
        control.completion = Some((request, result));
        Ok(())
    }
}
