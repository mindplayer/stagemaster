use super::ManagedWorker;
use stagemaster_install::Storage;
use stagemaster_package::{Entry, StepLabel};
use stagemaster_runtime::{Code, FrameInfo, Grant, Lease, PlaybackPolicy, Receipt, Request};

// Delegation keeps control leases, program semantics and all state transitions in Runtime.
impl<S: Storage, P: PlaybackPolicy> ManagedWorker<S, P> {
    #[must_use]
    pub fn catalog(&self) -> &[Entry] {
        self.runtime.catalog()
    }
    #[must_use]
    pub fn steps(&self) -> &[StepLabel] {
        self.runtime.steps()
    }
    /// Trusted policy updates only; not an IPC authorization flag.
    pub fn policy_mut(&mut self) -> &mut P {
        self.runtime.policy_mut()
    }
    /// # Errors
    /// Preserve runtime clock and playback failures.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), Code> {
        self.runtime.tick(now_ms)
    }
    /// # Errors
    /// Reject invalid grants, ownership conflicts and clocks before changing control.
    pub fn acquire(&mut self, grant: Grant, takeover: bool, now_ms: u64) -> Result<Lease, Code> {
        self.runtime.acquire(grant, takeover, now_ms)
    }
    /// # Errors
    /// Expired or replaced input authority cannot be renewed.
    pub fn renew(&mut self, lease: Lease, duration_ms: u64, now_ms: u64) -> Result<(), Code> {
        self.runtime.renew(lease, duration_ms, now_ms)
    }
    /// Releasing control does not stop a running program.
    /// # Errors
    /// Reject old leases and backwards clocks.
    pub fn release(&mut self, lease: Lease, now_ms: u64) -> Result<(), Code> {
        self.runtime.release(lease, now_ms)
    }
    /// # Errors
    /// Preserve runtime input validation, historical receipts and explicit maintenance intent.
    pub fn submit(&mut self, request: Request, now_ms: u64) -> Result<Receipt, Code> {
        self.runtime.submit(request, now_ms)
    }
    /// Logical frames only; this is never proof of physical transmission.
    /// # Errors
    /// Preserve core mapping errors; unavailable frames do not clear the buffer.
    pub fn render(&self, target: &mut [u8; 512]) -> Result<Option<FrameInfo>, Code> {
        self.runtime.render(target)
    }
}
