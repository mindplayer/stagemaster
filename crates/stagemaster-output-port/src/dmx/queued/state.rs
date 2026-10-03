use super::Fault;
use crate::{Event, Ticket};
use embassy_sync::waitqueue::WakerRegistration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Life {
    Fresh,
    Open,
    ProducerClosed,
    ConsumerClosed,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Frame(Ticket),
    Stop(Ticket),
}
pub(super) struct Frame {
    pub ticket: Ticket,
    pub slots: [u8; 512],
    pub until: u64,
}
// Fixed stack storage is intentional; boxing this variant would allocate per frame.
#[allow(clippy::large_enum_variant)]
pub(super) enum Work {
    Frame(Frame),
    Stop(Ticket),
}
impl Work {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Frame(f) => Kind::Frame(f.ticket),
            Self::Stop(t) => Kind::Stop(*t),
        }
    }
}
pub(super) struct State {
    pub life: Life,
    pub frame: Option<Frame>,
    pub active: Option<Kind>,
    pub stop: Option<Ticket>,
    pub completion: Option<Event>,
    pub fault: Option<Fault>,
    pub fault_pending: bool,
    pub wake: WakerRegistration,
}
impl State {
    pub const fn new() -> Self {
        Self {
            life: Life::Fresh,
            frame: None,
            active: None,
            stop: None,
            completion: None,
            fault: None,
            fault_pending: false,
            wake: WakerRegistration::new(),
        }
    }
    pub fn fail(&mut self, fault: Fault) {
        if self.fault.is_none() {
            self.fault = Some(fault);
            self.fault_pending = true;
        }
        self.frame = None;
        self.completion = None;
        self.wake.wake();
    }
    pub fn interrupted(&self, kind: Kind) -> bool {
        self.life != Life::Open
            || match kind {
                Kind::Frame(_) => self.stop.is_some() || self.fault.is_some(),
                Kind::Stop(t) => self.stop != Some(t),
            }
    }
    pub fn take(&mut self) -> Option<Work> {
        let work = if let Some(t) = self.stop {
            Work::Stop(t)
        } else {
            Work::Frame(self.frame.take()?)
        };
        self.active = Some(work.kind());
        Some(work)
    }
    pub fn finish(&mut self, kind: Kind, result: Option<Result<Event, Fault>>) {
        let current = self.active == Some(kind) && !self.interrupted(kind);
        self.active = None;
        if let Some(Err(fault)) = result {
            self.fail(fault);
        }
        if !current {
            return;
        }
        if matches!(kind, Kind::Stop(_)) {
            // Even a failed stop is consumed once; do not busy-loop on broken I/O.
            self.stop = None;
        }
        if let Some(Ok(event)) = result {
            let valid = match (kind, event) {
                (Kind::Frame(a), Event::Sent(b) | Event::Expired(b))
                | (Kind::Stop(a), Event::Quiet(b)) => a == b,
                _ => false,
            };
            if valid && self.completion.is_none() {
                self.completion = Some(event);
            } else {
                self.fail(Fault::Report);
            }
        }
    }
}
