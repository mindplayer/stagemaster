use super::{MediaPort, Worker};
use stagemaster_live::{Session, media::GroupKey};
use stagemaster_runtime::Code;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Termination {
    Ended,
    Failed(Code),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminationReceipt {
    pub serial: u64,
    /// The containing media state identifies the fixed group; only this playback generation ended.
    pub generation: u64,
    pub reason: Termination,
    pub result: Result<(), Code>,
}
pub(super) struct Request {
    serial: u64,
    key: GroupKey,
    reason: Termination,
}
impl MediaPort {
    /// Report a trusted provider's terminal state after stopping its actual source.
    /// # Errors
    /// Reject other groups, shutdown, a full terminal slot, contention or exhausted serials.
    pub fn terminate(&self, key: GroupKey, reason: Termination) -> Result<u64, Code> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(Code::State);
        }
        if !key.same_group(self.group.key) {
            return Err(Code::Selection);
        }
        let mut inbox = self.inbox.try_lock().map_err(|_| Code::Busy)?;
        if inbox.termination.is_some() {
            return Err(Code::Busy);
        }
        let serial = inbox.serial.checked_add(1).ok_or(Code::Exhausted)?;
        inbox.termination = Some(Request {
            serial,
            key,
            reason,
        });
        inbox.serial = serial;
        Ok(serial)
    }
}
impl Worker {
    pub(super) fn poll_termination(&mut self, session: &mut Session, now: u64) -> Result<(), Code> {
        let terminal = match self.inbox.try_lock() {
            Ok(mut inbox) => inbox.termination.take(),
            Err(_) => None,
        };
        if let Some(terminal) = terminal {
            let result = session.stop_media(terminal.key, now).map_err(|_| {
                if session.fault().is_some() {
                    Code::Playback
                } else {
                    Code::State
                }
            });
            self.termination = Some(TerminationReceipt {
                serial: terminal.serial,
                generation: terminal.key.generation(),
                reason: terminal.reason,
                result,
            });
        }
        if session.fault().is_some() {
            Err(Code::Playback)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "termination_tests.rs"]
mod tests;
