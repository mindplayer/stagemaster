use crate::{Code, Grant, Id, Lease, MAX_LEASE_MS, Owner, Receipt, Request};
pub(crate) struct Control {
    pub owner: Owner,
    pub receipt: Option<Receipt>,
}
impl Control {
    pub fn new(boot: Id, epoch: u64, grant: Grant, now_ms: u64) -> Result<Self, Code> {
        if grant.principal == [0; 16] || !(1..=MAX_LEASE_MS).contains(&grant.duration_ms) {
            return Err(Code::Identity);
        }
        let expires_ms = now_ms
            .checked_add(grant.duration_ms)
            .ok_or(Code::Exhausted)?;
        Ok(Self {
            owner: Owner {
                lease: Lease { boot, epoch },
                principal: grant.principal,
                origin: grant.origin,
                expires_ms,
                serial: 0,
            },
            receipt: None,
        })
    }
    pub fn request(&self, request: Request) -> Result<Option<Receipt>, Code> {
        if request.lease != self.owner.lease {
            return Err(Code::Lease);
        }
        if let Some(receipt) = self.receipt.filter(|r| r.request.serial == request.serial) {
            return if receipt.request == request {
                Ok(Some(receipt))
            } else {
                Err(Code::Sequence)
            };
        }
        if self.owner.serial.checked_add(1) != Some(request.serial) {
            return Err(Code::Sequence);
        }
        Ok(None)
    }
}
