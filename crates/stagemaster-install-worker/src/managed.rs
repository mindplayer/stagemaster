//! Compose runtime maintenance and serialized installation without a mutable escape hatch.
mod control;

use crate::{Command, Completion, Epoch, Error, Worker};
use stagemaster_install::{Installed, Installer, Phase, Storage};
use stagemaster_runtime::{
    Code, Maintenance, MaintenanceError, PlaybackPolicy, Quiescence, Runtime, State,
};

/// One owner on the storage executor. Authentication remains with the connection adapter;
/// physical quiescence remains with the output host. Neither is inferred from peer fields.
pub struct ManagedWorker<S: Storage, P> {
    worker: Worker<S>,
    runtime: Runtime<S::Snapshot, P>,
    permit: Option<Maintenance>,
}
impl<S: Storage, P: PlaybackPolicy> ManagedWorker<S, P> {
    /// # Errors
    /// Refuse invalid runtime budgets/identity or an installer with an existing transaction.
    pub fn new(
        installer: Installer<S>,
        now_ms: u64,
        budget: usize,
        policy: P,
    ) -> Result<Self, Error> {
        let boot = installer.transaction(1).boot;
        Ok(Self {
            runtime: Runtime::new(boot, now_ms, budget, policy).map_err(Error::Maintenance)?,
            worker: Worker::new(installer).map_err(Error::Protocol)?,
            permit: None,
        })
    }

    #[must_use]
    pub fn state(&self) -> State {
        self.runtime.state()
    }

    #[must_use]
    pub const fn quiescence_request(&self) -> Option<Quiescence> {
        self.runtime.quiescence_request()
    }

    /// Only the actual output host can acknowledge stopped output and drained queues.
    /// # Errors
    /// Reject obsolete/cancelled confirmations; never acquire a permit from a network claim.
    pub fn confirm_quiescent(&mut self, request: Quiescence, now_ms: u64) -> Result<(), Code> {
        let permit = self.runtime.confirm_quiescent(request, now_ms)?;
        self.permit = Some(permit);
        self.worker.observe(None);
        Ok(())
    }

    /// Revoking communication retains installation ownership and durable outcomes.
    pub fn observe(&mut self, live: Option<Epoch>) {
        self.worker
            .observe(if self.permit.is_some() { live } else { None });
    }

    /// Execute synchronous storage work only inside the current maintenance window.
    /// Invalid operation modes do not stop or take over playback.
    pub fn process(
        &mut self,
        command: Command,
        now_ms: u64,
        live: impl FnMut() -> Option<Epoch>,
    ) -> Completion {
        let epoch = command.epoch();
        let ready = self
            .runtime
            .tick(now_ms)
            .and_then(|()| self.permit.ok_or(Code::Mode));
        let result = ready.and_then(|permit| {
            self.runtime
                .with_maintenance(permit, || self.worker.process(command, live))
        });
        match result {
            Ok(completion) => completion,
            Err(error) => {
                self.worker.observe(None);
                Completion {
                    epoch,
                    result: Err(Error::Maintenance(error)),
                }
            }
        }
    }

    /// Bind the latest durable package, without choosing or starting a program.
    /// # Errors
    /// Incomplete, failed and uncertain transactions must first be cancelled/reconciled.
    /// Snapshot/clock/budget failures retain maintenance for an explicit retry.
    pub fn finish_maintenance(
        &mut self,
        now_ms: u64,
    ) -> Result<State, MaintenanceError<stagemaster_install::Error<S::Error>>> {
        self.runtime.tick(now_ms).map_err(MaintenanceError::State)?;
        let permit = self.permit.ok_or(MaintenanceError::State(Code::Mode))?;
        if self
            .worker
            .service
            .progress()
            .is_some_and(|progress| !matches!(progress.phase, Phase::Committed | Phase::Cancelled))
        {
            return Err(MaintenanceError::State(Code::Busy));
        }
        let state =
            self.runtime
                .finish_maintenance(permit, now_ms, || match self.worker.snapshot() {
                    Ok(snapshot) => Ok(Some(snapshot)),
                    Err(stagemaster_install::Error::Code(stagemaster_install::Code::Empty)) => {
                        Ok(None)
                    }
                    Err(error) => Err(error),
                })?;
        self.permit = None;
        self.worker.observe(None);
        Ok(state)
    }

    /// Read-only installed data for local inspection on this same executor.
    /// Retained readers can block later storage reuse; callers must drop them promptly.
    /// # Errors
    /// Preserve storage, missing package and package validation failures.
    pub fn snapshot(&self) -> Result<Installed<S::Snapshot>, stagemaster_install::Error<S::Error>> {
        self.worker.snapshot()
    }
}
