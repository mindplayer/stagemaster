//! Shared input authority. No program, I/O, policy, scheduler or implicit stop behavior.
use crate::{Code, Grant, Id, Lease, MAX_LEASE_MS, Owner, Receipt, Request};

pub struct Authority<A, S> {
    boot: Id,
    epoch: u64,
    now_ms: u64,
    owner: Option<Owner>,
    receipt: Option<Receipt<A, S>>,
}
impl<A: Clone + Eq, S: Clone> Authority<A, S> {
    /// # Errors
    /// Reject a missing boot identity; caller must use a fresh identity per lifetime.
    pub fn new(boot: Id, now_ms: u64) -> Result<Self, Code> {
        if boot == [0; 16] {
            return Err(Code::Identity);
        }
        Ok(Self {
            boot,
            epoch: 0,
            now_ms,
            owner: None,
            receipt: None,
        })
    }
    #[must_use]
    pub const fn owner(&self) -> Option<Owner> {
        self.owner
    }
    /// Expiry relinquishes input only. Scheduling/output remains the backend's responsibility.
    /// # Errors
    /// Backwards time changes nothing.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), Code> {
        if now_ms < self.now_ms {
            return Err(Code::Clock);
        }
        self.now_ms = now_ms;
        if self.owner.is_some_and(|o| now_ms >= o.expires_ms) {
            self.clear();
        }
        Ok(())
    }
    /// # Errors
    /// Reject busy authority, invalid grants or exhausted counters without replacing an owner.
    pub fn acquire(&mut self, grant: Grant, takeover: bool) -> Result<Lease, Code> {
        if self.owner.is_some() && !takeover {
            return Err(Code::Busy);
        }
        if grant.principal == [0; 16] || !(1..=MAX_LEASE_MS).contains(&grant.duration_ms) {
            return Err(Code::Identity);
        }
        let epoch = self.epoch.checked_add(1).ok_or(Code::Exhausted)?;
        let expires_ms = self
            .now_ms
            .checked_add(grant.duration_ms)
            .ok_or(Code::Exhausted)?;
        let lease = Lease {
            boot: self.boot,
            epoch,
        };
        self.owner = Some(Owner {
            lease,
            principal: grant.principal,
            origin: grant.origin,
            expires_ms,
            serial: 0,
        });
        self.receipt = None;
        self.epoch = epoch;
        Ok(lease)
    }
    /// # Errors
    /// Invalid durations and expired/replaced leases cannot renew authority.
    pub fn renew(&mut self, lease: Lease, duration_ms: u64) -> Result<(), Code> {
        if !(1..=MAX_LEASE_MS).contains(&duration_ms) {
            return Err(Code::Identity);
        }
        let expires = self
            .now_ms
            .checked_add(duration_ms)
            .ok_or(Code::Exhausted)?;
        self.owner
            .as_mut()
            .filter(|o| o.lease == lease)
            .ok_or(Code::Lease)?
            .expires_ms = expires;
        Ok(())
    }
    /// # Errors
    /// A stale release cannot revoke a newer owner.
    pub fn release(&mut self, lease: Lease) -> Result<(), Code> {
        if self.owner.is_none_or(|o| o.lease != lease) {
            return Err(Code::Lease);
        }
        self.clear();
        Ok(())
    }
    /// Trusted serialized backend: validate before applying, then complete exactly once.
    /// A duplicate returns its historical receipt and must not execute again.
    /// # Errors
    /// Wrong leases are refused; conflicting/skipped serials revoke the current input authority.
    pub fn check(&mut self, request: &Request<A>) -> Result<Option<Receipt<A, S>>, Code> {
        let owner = self.owner.ok_or(Code::Lease)?;
        if request.lease != owner.lease {
            return Err(Code::Lease);
        }
        if let Some(receipt) = self
            .receipt
            .as_ref()
            .filter(|r| r.request.serial == request.serial)
        {
            if receipt.request == *request {
                return Ok(Some(receipt.clone()));
            }
        } else if owner.serial.checked_add(1) == Some(request.serial) {
            return Ok(None);
        }
        self.clear();
        Err(Code::Sequence)
    }
    /// Record a business result and snapshot after updating the accepted serial.
    /// State/action clones must be bounded and nonblocking; this container itself allocates nothing.
    /// # Errors
    /// Caller must keep authority unchanged between check, apply and completion.
    pub fn complete(
        &mut self,
        request: Request<A>,
        result: Result<(), Code>,
        snapshot: impl FnOnce(Option<Owner>) -> S,
    ) -> Result<Receipt<A, S>, Code> {
        if let Some(receipt) = self.check(&request)? {
            return Ok(receipt);
        }
        self.owner.as_mut().ok_or(Code::Lease)?.serial = request.serial;
        let receipt = Receipt {
            request,
            result,
            state: snapshot(self.owner),
        };
        self.receipt = Some(receipt.clone());
        Ok(receipt)
    }
    fn clear(&mut self) {
        self.owner = None;
        self.receipt = None;
    }
}
