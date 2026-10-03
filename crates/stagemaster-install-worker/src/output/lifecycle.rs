use super::{Error, LocalOutput};
use crate::ManagedWorker;
use stagemaster_install::Storage;
use stagemaster_output_port::{Driver, Phase};
use stagemaster_runtime::{Mode, PlaybackPolicy};

impl<D: Driver> LocalOutput<D> {
    /// Poll independently of frame generation, including during stop/maintenance.
    /// This is the only assembly path that confirms the worker's output quiescence.
    /// # Errors
    /// Faults latch. Late genuine Quiet can still be observed but cannot grant maintenance.
    pub fn service<S: Storage, P: PlaybackPolicy>(
        &mut self,
        worker: &mut ManagedWorker<S, P>,
        now: u64,
    ) -> Result<(), Error> {
        let state = worker.state();
        if state.boot != self.source.id {
            return Err(self.fail(Error::Snapshot, now));
        }
        if self.fault.is_none()
            && (state.mode != Mode::Operation
                || state.loaded.is_none()
                || (state.instance.is_none() && self.granted.is_none()))
        {
            // Revoke before poll: it can consume a queued Quiet and dispatch a
            // pending frame. A cancelled initial start must not gain a permit.
            self.disable(now)?;
        }
        let result = self.port.poll(now);
        if let Some(error) = self.fault {
            return Err(error);
        }
        result.map_err(|e| self.fail(Error::Output(e), now))?;
        if state.mode != Mode::Operation {
            if let Some(request) = worker.quiescence_request()
                && self.port.state().quiet
            {
                let mut confirmed = Ok(());
                self.port
                    .with_quiescent(|| {
                        confirmed = worker.confirm_quiescent(request, now);
                    })
                    .map_err(|e| self.fail(Error::Output(e), now))?;
                confirmed.map_err(|e| self.fail(Error::Runtime(e), now))?;
            }
            return Ok(());
        }
        let output = self.port.state();
        if self.granted.is_some() && output.permit.is_none() {
            self.wanted = false;
        }
        self.granted = output.permit;
        if let Some(instance) = state.instance
            && self.seen != Some(instance)
        {
            if instance.boot != self.source.id {
                return Err(self.fail(Error::Snapshot, now));
            }
            self.seen = Some(instance);
            self.wanted = true;
        }
        if state.loaded.is_none() || (state.instance.is_none() && output.permit.is_none()) {
            self.wanted = false;
        }
        if !self.wanted {
            return self.disable(now);
        }
        if matches!(output.phase, Phase::Idle | Phase::Unconfirmed) {
            self.port
                .select(self.source, false, now)
                .map_err(|e| self.fail(Error::Output(e), now))?;
        }
        Ok(())
    }
}
