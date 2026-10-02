//! Multi-source execution adapter for the existing independent host, with shared input authority.
mod commands;
mod types;
use stagemaster_live::Session;
use stagemaster_runtime::{Code, Grant, Lease, Receipt, Request, authority::Authority};
use stagemaster_runtime_host::Backend;
pub use types::{Action, FrameInfo, Live, Patch, State};

pub struct LiveBackend {
    session: Session,
    control: Authority<Action, State>,
    revision: u64,
}
impl LiveBackend {
    /// Transfer a pristine prepared group; no other object retains mutable execution access.
    /// # Errors
    /// Reject already-used or faulted groups. This is not proof of authentication or a commercial grant.
    pub fn new(session: Session) -> Result<Self, Code> {
        if !session.is_pristine() {
            return Err(Code::State);
        }
        let control = Authority::new(session.boot(), session.observed_ms())?;
        Ok(Self {
            session,
            control,
            revision: 0,
        })
    }
    fn apply(&mut self, action: &Action, now: u64) -> Result<(), Code> {
        let (source, result) = match action {
            Action::Control { source, command } => {
                (*source, self.session.control(*source, *command, now))
            }
            Action::Patch { source, patch } => {
                (*source, self.session.patch(*source, patch.values(), now))
            }
            Action::Level { source, level } => {
                (*source, self.session.set_level(*source, *level, now))
            }
        };
        result.map_err(|_| {
            if self.session.fault().is_some() {
                Code::Playback
            } else if self.session.sources().any(|s| s.key == source) {
                Code::State
            } else {
                Code::Selection
            }
        })
    }
}
impl Backend for LiveBackend {
    type Profile = Live;
    fn is_prepared(&self) -> bool {
        self.session.is_pristine() && self.control.owner().is_none()
    }
    fn observed_ms(&self) -> u64 {
        self.session.observed_ms()
    }
    fn state(&self) -> State {
        let mut sources = [None; 64];
        for (slot, source) in sources.iter_mut().zip(self.session.sources()) {
            *slot = Some(source);
        }
        State {
            boot: self.session.boot(),
            revision: self.revision,
            observed_ms: self.session.observed_ms(),
            layout: self.session.layout_id(),
            owner: self.control.owner(),
            sources,
            fault: self.session.fault().is_some(),
        }
    }
    fn tick(&mut self, now: u64) -> Result<(), Code> {
        if now < self.session.observed_ms() {
            return Err(Code::Clock);
        }
        self.session.tick(now).map_err(|_| Code::Playback)?;
        self.control.tick(now)
    }
    fn render(&self, slots: &mut [u8; 512]) -> Result<Option<FrameInfo>, Code> {
        if self.session.fault().is_some() {
            return Err(Code::Playback);
        }
        Ok(self.session.frame().map(|frame| {
            *slots = frame.slots;
            FrameInfo {
                boot: self.session.boot(),
                revision: self.revision,
                layout: self.session.layout_id(),
                sampled_ms: frame.sampled_ms,
                sequence: frame.sequence,
                universe: frame.universe,
            }
        }))
    }
    fn acquire(&mut self, grant: Grant, takeover: bool, now: u64) -> Result<Lease, Code> {
        self.acquire_control(grant, takeover, now)
    }
    fn submit(
        &mut self,
        request: Request<Action>,
        now: u64,
    ) -> Result<Receipt<Action, State>, Code> {
        self.submit_control(request, now)
    }
    fn renew(&mut self, lease: Lease, duration: u64, now: u64) -> Result<(), Code> {
        self.tick(now)?;
        self.control.renew(lease, duration)
    }
    fn release(&mut self, lease: Lease, now: u64) -> Result<(), Code> {
        self.tick(now)?;
        self.control.release(lease)
    }
}
