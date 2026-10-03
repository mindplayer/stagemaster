mod job;
mod observation;
mod output;
mod owner;
mod setup;
pub(crate) mod wire;
pub(crate) use output::OutputKind;
pub(crate) use owner::Owner;
pub(crate) use setup::Setup;
use stagemaster_audio::Transport;
use stagemaster_live::media::{GroupKey, Preparer};
use stagemaster_live_host::{
    Live,
    media::{LocalClock, MediaPort},
};
use stagemaster_project::Document;
use stagemaster_runtime_host::{Clock, Observer};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

struct Runner {
    transport: Transport,
    software: Option<output::SoftwareOutput>,
    doc: Document,
    prepare: Preparer,
    port: MediaPort,
    observer: Observer<Live>,
    clock: Clock,
    mapping: LocalClock,
    cancel: Arc<AtomicBool>,
    view: Arc<Mutex<owner::View>>,
    active: Option<GroupKey>,
    request: Option<stagemaster_live_host::media::ControlRequest>,
    staged: Option<stagemaster_live_host::media::Activation>,
    pending_sample: Option<(stagemaster_live::media::Sample, stagemaster_time::Mapping)>,
    seen: Option<stagemaster_live_host::media::ControlTicket>,
    last_sample: u64,
    terminal: Option<u64>,
    pending_end: Option<(GroupKey, stagemaster_live_host::media::Termination)>,
}
