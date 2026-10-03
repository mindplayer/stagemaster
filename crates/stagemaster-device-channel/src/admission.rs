use stagemaster_device_info::{Description, capability};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_runtime_protocol::Ready;

pub(crate) enum Admission {
    Installation(ApplicationReceipt),
    Runtime(Ready),
}
impl Admission {
    pub fn matches(&self, desc: Description) -> bool {
        let (device, boot, diagnostic, capability) = match self {
            Self::Installation(value) => (
                value.device,
                value.boot,
                value.diagnostic,
                capability::INSTALLATION,
            ),
            Self::Runtime(value) => (
                value.peer.device,
                value.peer.boot,
                value.peer.connection,
                capability::RUNTIME_APPLICATION,
            ),
        };
        desc.device == device
            && desc.boot == boot
            && desc.session == diagnostic
            && desc.declares(capability)
            && desc.authentication == stagemaster_device_session::AUTHENTICATION
    }
}
