//! Serialized runtime application boundary and bounded wire adapter. No scheduler or output driver.
mod dispatch;
mod error;
mod lifecycle;
mod wire;
pub use error::Error;
pub use stagemaster_runtime_protocol::{
    Detail, Failure, Operation, Program, Reply, Request, Step, Text,
};

use crate::ManagedWorker;
use stagemaster_device_auth::application::{Grant, Scope};
use stagemaster_install::Storage;
use stagemaster_runtime::{Lease, PlaybackPolicy};

/// One admitted connection, an optional input lease and one immutable last receipt.
/// Keep on the same serialized worker as `ManagedWorker`. Authentication stays outside.
/// Keep exactly one instance per admitted session; never rebuild it to reset request IDs.
pub struct Connection {
    grant: Grant,
    lease: Option<Lease>,
    last: Option<Reply>,
    last_ms: u64,
    closed: bool,
    wire_ready: bool,
}
impl Connection {
    /// Validate current application access without touching runtime state or maintenance.
    /// `live` must recheck the real Session (or an equivalent live revocation source),
    /// never return a cached Grant. The timestamp belongs to the trusted device clock.
    /// # Errors
    /// Refuses a foreign boot, stale access, clock rollback or installation-only access.
    pub fn open<S: Storage, P: PlaybackPolicy>(
        device: &ManagedWorker<S, P>,
        now: u64,
        mut live: impl FnMut(u64) -> Option<Grant>,
    ) -> Result<Self, Error> {
        let grant = live(now).ok_or(Error::Obsolete)?;
        if now >= grant.expires_at() {
            return Err(Error::Obsolete);
        }
        if grant.context().boot != device.state().boot {
            return Err(Error::Identity);
        }
        if now < device.state().observed_ms {
            return Err(Error::Clock);
        }
        if !grant.permissions().contains(Scope::Observe)
            && !grant.permissions().contains(Scope::Control)
        {
            return Err(Error::Denied);
        }
        Ok(Self {
            grant,
            lease: None,
            last: None,
            last_ms: now,
            closed: false,
            wire_ready: false,
        })
    }

    /// Process one request on the runtime owner. Sample fresh time/access again after
    /// potentially slow storage work. A failure after execution is NOT a rollback.
    /// The host must independently tick Runtime and poll connection revocation while idle.
    /// # Errors
    /// Stale authority, protocol sequence errors and permission failures close input only.
    pub fn process<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        request: Request,
        mut clock: impl FnMut() -> u64,
        mut live: impl FnMut(u64) -> Option<Grant>,
    ) -> Result<Reply, Error> {
        let now = clock();
        self.poll(device, now, &mut live)?;
        // A misrouted old request cannot evict this connection's current input lease.
        if request.session != self.grant.session() {
            return Err(Error::Identity);
        }
        let result = self.process_current(device, request, now);
        let reply = match result {
            Ok(reply) => reply,
            Err(error) => {
                self.close(device).map_err(Error::Runtime)?;
                return Err(error);
            }
        };
        self.poll(device, clock(), live)?;
        Ok(reply)
    }

    fn process_current<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        request: Request,
        now: u64,
    ) -> Result<Reply, Error> {
        self.authorize(request.operation)?;
        if let Some(last) = self.last.as_ref().filter(|r| r.request.id == request.id) {
            return if last.request == request {
                Ok(*last)
            } else {
                Err(Error::Sequence)
            };
        }
        if self
            .last
            .as_ref()
            .map_or(Some(1), |r| r.request.id.checked_add(1))
            != Some(request.id)
        {
            return Err(Error::Sequence);
        }
        let result = if request.operation != Operation::Status
            && request.expected_revision != device.state().revision
        {
            Err(Failure::Runtime(stagemaster_runtime::Code::Revision))
        } else {
            self.apply(device, request.operation, now)
        };
        let reply = Reply {
            request,
            result,
            state: device.state(),
            program_count: u16::try_from(device.catalog().len()).map_err(|_| Error::Identity)?,
            step_count: u16::try_from(device.steps().len()).map_err(|_| Error::Identity)?,
        };
        self.last = Some(reply);
        Ok(reply)
    }

    fn authorize(&self, operation: Operation) -> Result<(), Error> {
        use stagemaster_runtime::Action;
        let required = match operation {
            Operation::Status | Operation::Catalog { .. } | Operation::Step { .. } => {
                Scope::Observe
            }
            _ => Scope::Control,
        };
        let maintenance = matches!(
            operation,
            Operation::FinishMaintenance
                | Operation::Apply(Action::BeginMaintenance | Action::CancelMaintenance)
        );
        if self.grant.permissions().contains(required)
            && (!maintenance || self.grant.permissions().contains(Scope::Installation))
        {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
}
