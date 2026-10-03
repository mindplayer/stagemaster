use stagemaster_live::media::GroupKey;
use stagemaster_runtime::Code;

#[derive(Clone, Copy, Debug)]
pub struct ControlSpec {
    pub group: [u8; 16],
    pub duration_ms: u64,
    pub timeout_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaCommand {
    Play,
    Pause,
    Stop,
    Seek {
        position_ms: u64,
        playing: bool,
    },
    ExitLoop {
        instance: u64,
        region: usize,
        pass: u64,
        requested: bool,
    },
}

/// Granted by the original input authority; neither an operator lease nor a physical-output permit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTicket {
    pub(super) group: GroupKey,
    pub(super) serial: u64,
}
impl ControlTicket {
    #[must_use]
    pub const fn serial(self) -> u64 {
        self.serial
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlRequest {
    pub ticket: ControlTicket,
    pub command: MediaCommand,
    pub deadline_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlFailure {
    Provider(Code),
    TimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlState {
    pub request: ControlRequest,
    /// None means accepted, awaiting provider completion. It never means audio has already changed.
    pub result: Option<Result<(), ControlFailure>>,
}
