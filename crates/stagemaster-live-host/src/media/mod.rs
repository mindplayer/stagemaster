//! Fixed-capacity local media handoff; operator control remains in the original host queue.
mod control;
mod local_clock;
mod port;
mod slots;
mod worker;
pub use control::{
    ControlFailure, ControlRequest, ControlSpec, ControlState, ControlTicket, MediaCommand,
};

pub use local_clock::LocalClock;
pub use port::MediaPort;
pub(crate) use slots::Worker;
pub use slots::{Activation, ObservationReceipt, Reclaimed};
use stagemaster_live::{Session, media::GroupInfo};
use stagemaster_runtime::Code;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaState {
    pub group: GroupInfo,
    pub observation: Option<ObservationReceipt>,
    pub control: Option<ControlState>,
}

pub(crate) fn prepare(session: &Session) -> Result<(Vec<Worker>, Vec<MediaPort>), Code> {
    if !session.is_pristine() {
        return Err(Code::State);
    }
    Ok(session.media_groups().map(slots::pair).unzip())
}

#[cfg(test)]
mod tests;
