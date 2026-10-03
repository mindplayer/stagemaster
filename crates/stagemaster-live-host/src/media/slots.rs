use super::MediaPort;
use stagemaster_live::media::{GroupInfo, GroupKey, Prepared, Sample};
use stagemaster_runtime::Code;
use stagemaster_time::Mapping;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Opaque scalar capability. Cloning a command or receipt never clones a compiled plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activation {
    pub(super) group: GroupKey,
    pub(super) serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationReceipt {
    pub serial: u64,
    /// Fixed identity belongs to the containing `MediaState`; retain the input playback generation.
    pub generation: u64,
    pub sample_sequence: u64,
    pub result: Result<(), Code>,
}
/// Reclaim on the preparation thread; contains replaced players even after successful activation.
pub struct Reclaimed {
    pub prepared: Prepared,
    /// None means withdrawn before business application, not rollback of an earlier activation.
    pub result: Option<Result<(), Code>>,
}
pub(super) struct Entry {
    pub ticket: Activation,
    pub prepared: Prepared,
    pub sample: Sample,
    pub mapping: Mapping,
    pub result: Option<Result<(), Code>>,
    pub request: Option<super::ControlTicket>,
}
#[derive(Default)]
pub(super) struct Staging {
    pub serial: u64,
    pub entry: Option<Entry>,
}
#[derive(Clone, Copy)]
pub(super) struct Update {
    pub serial: u64,
    pub key: GroupKey,
    pub sample: Sample,
    pub mapping: Mapping,
}
#[derive(Default)]
pub(super) struct Inbox {
    pub serial: u64,
    pub latest: Option<Update>,
    pub termination: Option<super::termination::Request>,
}
pub(crate) struct Worker {
    pub(super) key: GroupKey,
    pub(super) staging: Arc<Mutex<Staging>>,
    pub(super) inbox: Arc<Mutex<Inbox>>,
    pub receipt: Option<ObservationReceipt>,
    pub termination: Option<super::TerminationReceipt>,
    pub(super) alive: Arc<AtomicBool>,
    pub(super) control: Option<super::control::Lane>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
    }
}
pub(super) fn pair(group: GroupInfo) -> (Worker, MediaPort) {
    let staging = Arc::new(Mutex::new(Staging::default()));
    let inbox = Arc::new(Mutex::new(Inbox::default()));
    let alive = Arc::new(AtomicBool::new(true));
    (
        Worker {
            key: group.key,
            staging: staging.clone(),
            inbox: inbox.clone(),
            receipt: None,
            termination: None,
            alive: alive.clone(),
            control: None,
        },
        MediaPort {
            group,
            staging,
            inbox,
            alive,
            control: None,
        },
    )
}
