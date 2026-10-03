mod provider;
mod registration;
#[cfg(test)]
mod tests;
mod types;
mod worker;
use stagemaster_live::media::GroupKey;
use stagemaster_runtime::Code;
use std::sync::{Arc, Mutex};
pub use types::{
    ControlFailure, ControlRequest, ControlSpec, ControlState, ControlTicket, MediaCommand,
};

#[derive(Default)]
pub(crate) struct Mailbox {
    pub serial: u64,
    pub state: Option<ControlState>,
    pub completion: Option<(ControlTicket, Result<(), Code>)>,
    pub activated: bool,
}
pub(crate) struct Lane {
    pub spec: ControlSpec,
    pub slot: Arc<Mutex<Mailbox>>,
    pub state: Option<ControlState>,
}
impl Mailbox {
    fn pending(&self, ticket: ControlTicket) -> Result<ControlRequest, Code> {
        self.state
            .filter(|s| s.request.ticket == ticket && s.result.is_none())
            .map(|s| s.request)
            .ok_or(Code::State)
    }
}
impl Lane {
    pub fn admit(&mut self, group: GroupKey, command: MediaCommand, now: u64) -> Result<(), Code> {
        if let MediaCommand::Seek { position_ms, .. } = command
            && position_ms > self.spec.duration_ms
        {
            return Err(Code::State);
        }
        let mut slot = self.slot.try_lock().map_err(|_| Code::Busy)?;
        let serial = slot.serial.checked_add(1).ok_or(Code::Exhausted)?;
        let deadline_ms = now.checked_add(self.spec.timeout_ms).ok_or(Code::Clock)?;
        let state = ControlState {
            request: ControlRequest {
                ticket: ControlTicket { group, serial },
                command,
                deadline_ms,
            },
            result: None,
        };
        slot.serial = serial;
        slot.state = Some(state);
        slot.completion = None;
        slot.activated = false;
        self.state = Some(state);
        Ok(())
    }
}
