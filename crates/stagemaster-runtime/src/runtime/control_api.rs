use crate::{Code, Grant, Lease, PlaybackPolicy, Receipt, Request, Runtime};
use stagemaster_package::ReadAt;

impl<R: ReadAt, P: PlaybackPolicy> Runtime<R, P> {
    /// Claim verified input authority; takeover is an explicit trusted-host decision.
    /// # Errors
    /// Refuse invalid grants, active ownership without takeover, clocks or exhausted counters.
    pub fn acquire(&mut self, grant: Grant, takeover: bool, now_ms: u64) -> Result<Lease, Code> {
        self.tick(now_ms)?;
        if self.control.owner().is_some() && !takeover {
            return Err(Code::Busy);
        }
        let revision = self.next_revision()?;
        let lease = self.control.acquire(grant, takeover)?;
        self.revision = revision;
        Ok(lease)
    }
    /// # Errors
    /// Expired/replaced leases cannot be renewed or resurrected.
    pub fn renew(&mut self, lease: Lease, duration_ms: u64, now_ms: u64) -> Result<(), Code> {
        self.tick(now_ms)?;
        self.control.renew(lease, duration_ms)
    }
    /// Disconnect relinquishes input authority only, never stopping the current program.
    /// # Errors
    /// An old disconnect cannot revoke a newer owner's lease.
    pub fn release(&mut self, lease: Lease, now_ms: u64) -> Result<(), Code> {
        self.tick(now_ms)?;
        self.control.release(lease)
    }
    /// Return immutable historical receipts while continuous playback advances.
    /// # Errors
    /// Protocol/lease/clock errors have no receipt; business refusals are cached.
    pub fn submit(&mut self, request: Request, now_ms: u64) -> Result<Receipt, Code> {
        self.tick(now_ms)?;
        if let Some(receipt) = self.control.check(&request)? {
            return Ok(receipt);
        }
        let result = if request.expected_revision != self.revision {
            Err(Code::Revision)
        } else if let Ok(next) = self.next_revision() {
            let result = self.apply(request.action, next);
            self.revision = next;
            result
        } else {
            Err(Code::Exhausted)
        };
        let mut state = self.state();
        self.control.complete(request, result, |owner| {
            state.owner = owner;
            state
        })
    }
}
