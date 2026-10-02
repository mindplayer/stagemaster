//! Trusted execution adapters. All worker callbacks must be bounded and free of blocking I/O.
use crate::Action;
use stagemaster_package::ReadAt;
use stagemaster_runtime::{
    Code, FrameInfo, Grant, Lease, Mode, PlaybackPolicy, Receipt, Request, Runtime, State, Status,
};

pub trait Profile: Copy + std::fmt::Debug + Eq + Send + Sync + 'static {
    type Action: Clone + std::fmt::Debug + Send + 'static;
    type State: Copy + std::fmt::Debug + Eq + Send + Sync + 'static;
    type Receipt: Clone + std::fmt::Debug + Send + 'static;
    type FrameInfo: Copy + std::fmt::Debug + Eq + Send + Sync + 'static;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Device;
impl Profile for Device {
    type Action = Action;
    type State = State;
    type Receipt = Receipt;
    type FrameInfo = FrameInfo;
}
/// Ownership is transferred to one worker. No callback may start its own clock/thread or read media.
/// Preparation must establish all required resources before `is_prepared` can return true.
pub trait Backend: Send + 'static {
    type Profile: Profile;
    fn is_prepared(&self) -> bool;
    fn observed_ms(&self) -> u64;
    fn state(&self) -> <Self::Profile as Profile>::State;
    /// # Errors
    /// Report clock or execution failures, which terminate this host and revoke its observation.
    fn tick(&mut self, now_ms: u64) -> Result<(), Code>;
    /// # Errors
    /// Report invalid output mappings, never return a partial frame as successful.
    fn render(
        &self,
        slots: &mut [u8; 512],
    ) -> Result<Option<<Self::Profile as Profile>::FrameInfo>, Code>;
    /// # Errors
    /// Refuse invalid grants or unapproved takeover; grants are verified by the trusted caller.
    fn acquire(&mut self, grant: Grant, takeover: bool, now_ms: u64) -> Result<Lease, Code>;
    /// # Errors
    /// Refuse stale leases, bad order or invalid clocks. Business outcomes belong in receipts.
    fn submit(
        &mut self,
        request: Request<<Self::Profile as Profile>::Action>,
        now_ms: u64,
    ) -> Result<<Self::Profile as Profile>::Receipt, Code>;
    /// # Errors
    /// Expired/replaced leases cannot be resurrected.
    fn renew(&mut self, lease: Lease, duration_ms: u64, now_ms: u64) -> Result<(), Code>;
    /// # Errors
    /// Old releases cannot revoke new owners; releasing input does not stop playback.
    fn release(&mut self, lease: Lease, now_ms: u64) -> Result<(), Code>;
}
impl<R: ReadAt + Send + 'static, P: PlaybackPolicy + Send + 'static> Backend for Runtime<R, P> {
    type Profile = Device;
    fn is_prepared(&self) -> bool {
        let s = self.state();
        s.mode == Mode::Operation
            && s.loaded.is_some()
            && s.selected == s.loaded
            && s.status == Some(Status::Idle)
            && s.instance.is_none()
            && s.owner.is_none()
    }
    fn observed_ms(&self) -> u64 {
        self.state().observed_ms
    }
    fn state(&self) -> State {
        self.state()
    }
    fn tick(&mut self, now: u64) -> Result<(), Code> {
        self.tick(now)
    }
    fn render(&self, slots: &mut [u8; 512]) -> Result<Option<FrameInfo>, Code> {
        self.render(slots)
    }
    fn acquire(&mut self, grant: Grant, takeover: bool, now: u64) -> Result<Lease, Code> {
        self.acquire(grant, takeover, now)
    }
    fn submit(&mut self, request: Request<Action>, now: u64) -> Result<Receipt, Code> {
        self.submit(
            Request {
                lease: request.lease,
                serial: request.serial,
                expected_revision: request.expected_revision,
                action: request.action.into(),
            },
            now,
        )
    }
    fn renew(&mut self, lease: Lease, duration: u64, now: u64) -> Result<(), Code> {
        self.renew(lease, duration, now)
    }
    fn release(&mut self, lease: Lease, now: u64) -> Result<(), Code> {
        self.release(lease, now)
    }
}
