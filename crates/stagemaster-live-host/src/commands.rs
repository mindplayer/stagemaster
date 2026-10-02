use crate::{Action, LiveBackend, State};
use stagemaster_runtime::{Code, Grant, Lease, Receipt, Request};
use stagemaster_runtime_host::Backend;

impl LiveBackend {
    pub(super) fn acquire_control(
        &mut self,
        grant: Grant,
        takeover: bool,
        now: u64,
    ) -> Result<Lease, Code> {
        self.tick(now)?;
        if self.control.owner().is_some() && !takeover {
            return Err(Code::Busy);
        }
        let next = self.revision.checked_add(1).ok_or(Code::Exhausted)?;
        let lease = self.control.acquire(grant, takeover)?;
        self.revision = next;
        Ok(lease)
    }
    pub(super) fn submit_control(
        &mut self,
        request: Request<Action>,
        now: u64,
    ) -> Result<Receipt<Action, State>, Code> {
        self.tick(now)?;
        if let Some(receipt) = self.control.check(&request)? {
            return Ok(receipt);
        }
        let result = if request.expected_revision != self.revision {
            Err(Code::Revision)
        } else if let Some(next) = self.revision.checked_add(1) {
            let result = self.apply(&request.action, now);
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
