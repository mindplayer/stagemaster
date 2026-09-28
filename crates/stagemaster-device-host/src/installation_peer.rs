use crate::{DeviceDescription, Problem, ProblemCode as C, description::hex};
use stagemaster_transfer::{Frame, MAX_FRAME_BYTES, Request, VERSION};

/// Installation session admitted by the native adapter, not an IPC permission claim.
/// The bonded GATT adapter correlates the device-authorized receipt and subscribes
/// a bounded receiver. The device independently checks its actual security and
/// authority on every dispatch; this value is not remote firmware attestation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallationPeer {
    pub device: [u8; 16],
    pub boot: [u8; 16],
    pub session: [u8; 16],
    pub authentication: u16,
    /// Maximum fragment payload accepted in both directions on this connection.
    pub fragment_bytes: u16,
    pub message_bytes: u16,
}

impl InstallationPeer {
    pub(crate) fn validate(self, description: Option<&DeviceDescription>) -> Result<(), Problem> {
        let valid = description.is_some_and(|d| {
            d.installation_declared
                && d.device_id == hex(&self.device)
                && d.boot_id == hex(&self.boot)
                && d.authentication_method == self.authentication
                && d.limits.transfer_version == u16::from(VERSION)
                && d.limits.message_bytes == self.message_bytes
        }) && self.authentication != 0
            && self.session != [0; 16]
            && self.fragment_bytes > 0
            && self.fragment_bytes <= self.message_bytes
            && (9..=MAX_FRAME_BYTES).contains(&usize::from(self.message_bytes));
        if valid {
            Ok(())
        } else {
            Err(Problem::new(C::Installation))
        }
    }

    pub(crate) fn request(self, frame: &Frame) -> Result<(), Problem> {
        let request = Request::decode(frame.bytes()).map_err(wire_error)?;
        if request.link != self.session || frame.bytes().len() > usize::from(self.message_bytes) {
            return Err(Problem::new(C::Protocol));
        }
        Ok(())
    }
}

pub(crate) fn wire_error(error: stagemaster_transfer::Error) -> Problem {
    Problem::new(C::Protocol).detail(error.to_string())
}
