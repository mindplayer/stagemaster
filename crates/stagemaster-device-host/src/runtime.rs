//! Native runtime observations; these facts never grant device authority.
use crate::{DeviceDescription, Problem, ProblemCode as C, description::hex};
use stagemaster_runtime_protocol::{Operation, Ready, Response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeIntent {
    pub operation: Operation,
    pub expected_revision: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    /// Epoch of the connection that owns this history, even during a later scan.
    pub connection_epoch: Option<u32>,
    /// Present only for the current fresh connection; not physical output evidence.
    pub peer: Option<Ready>,
    /// Retained on cancellation/failure after dispatch begins; execution is uncertain.
    pub pending: Option<RuntimeIntent>,
    /// Historical result, never a current-online flag. Cleared on a new connection.
    pub last_response: Option<Response>,
}

pub(crate) fn validate(peer: Ready, desc: Option<&DeviceDescription>) -> Result<(), Problem> {
    let valid = desc.is_some_and(|d| {
        d.runtime_declared
            && d.device_id == hex(&peer.peer.device)
            && d.boot_id == hex(&peer.peer.boot)
            && d.diagnostic_session == peer.peer.connection
            && d.authentication_method == stagemaster_device_session::AUTHENTICATION
            && d.limits.message_bytes == peer.message_bytes
    });
    if valid && peer.access.observe && peer.encode().is_ok() {
        Ok(())
    } else {
        Err(Problem::new(C::Runtime))
    }
}
pub(crate) fn wire(error: stagemaster_device_channel::Error) -> Problem {
    use stagemaster_device_channel::Error as E;
    match error {
        E::Denied => Problem::new(C::Runtime),
        E::Timeout => Problem::new(C::Timeout),
        E::Closed => Problem::new(C::Lost),
        E::Transport(detail) => Problem::new(C::Lost).detail(detail),
        E::Protocol(detail) => Problem::new(C::Protocol).detail(detail),
        E::Bounds => Problem::new(C::Protocol).detail("设备运行消息超出容量".into()),
    }
}
