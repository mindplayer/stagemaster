//! Local player assembly; Runtime keeps program semantics, Port keeps output authority.
mod lifecycle;
use crate::ManagedWorker;
use stagemaster_install::Storage;
use stagemaster_output_port::{Config, Driver, Permit, Port, Sample, Source, SourceKind, State};
use stagemaster_runtime::{FrameInfo, Instance, Mode, PlaybackPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Output(stagemaster_output_port::Code),
    Runtime(stagemaster_runtime::Code),
    Snapshot,
}

/// One boot-bound runtime source and output port. No transport or file parsing.
pub struct LocalOutput<D: Driver> {
    port: Port<D>,
    source: Source,
    seen: Option<Instance>,
    granted: Option<Permit>,
    wanted: bool,
    serial: u64,
    fault: Option<Error>,
}
impl<D: Driver> LocalOutput<D> {
    /// # Errors
    /// Preserve the original port's identity and time-budget validation.
    pub fn new(config: Config, driver: D, now: u64) -> Result<Self, Error> {
        Ok(Self {
            source: Source {
                id: config.boot,
                kind: SourceKind::Local,
            },
            port: Port::new(config, driver, now).map_err(Error::Output)?,
            seen: None,
            granted: None,
            wanted: false,
            serial: 0,
            fault: None,
        })
    }
    #[must_use]
    pub fn state(&self) -> State {
        self.port.state()
    }
    #[must_use]
    pub const fn fault(&self) -> Option<Error> {
        self.fault
    }

    /// Explicitly withdraw physical output intent; the same instance cannot rearm it.
    /// # Errors
    /// Shutdown admission is not Quiet; preserve terminal faults and continue service.
    pub fn disable(&mut self, now: u64) -> Result<(), Error> {
        self.wanted = false;
        self.port
            .shutdown(now)
            .map(|_| ())
            .map_err(|e| self.fail(Error::Output(e), now))
    }

    /// Publish only an immediate render(Some) result from this worker. False means
    /// no current output permit, not a physical completion or an implicit arm.
    /// # Errors
    /// Wrong boot/revision/program/instance/time, output fault or counter exhaustion.
    pub fn publish<S: Storage, P: PlaybackPolicy>(
        &mut self,
        worker: &mut ManagedWorker<S, P>,
        info: FrameInfo,
        slots: &[u8; 512],
        now: u64,
    ) -> Result<bool, Error> {
        self.service(worker, now)?;
        let state = worker.state();
        if state.boot != self.source.id
            || info.boot != state.boot
            || state.mode != Mode::Operation
            || info.revision != state.revision
            || Some(info.program) != state.loaded
            || info.instance != state.instance
            || info.sampled_ms != state.observed_ms
        {
            return Err(self.fail(Error::Snapshot, now));
        }
        let Some(permit) = self.port.state().permit.filter(|_| self.wanted) else {
            return Ok(false);
        };
        self.serial = self.serial.checked_add(1).ok_or_else(|| {
            self.fail(Error::Output(stagemaster_output_port::Code::Exhausted), now)
        })?;
        self.port
            .submit(
                permit,
                Sample {
                    serial: self.serial,
                    sampled_ms: info.sampled_ms,
                    universe: info.universe,
                    slots,
                },
                now,
            )
            .map_err(|e| self.fail(Error::Output(e), now))?;
        self.port
            .poll(now)
            .map_err(|e| self.fail(Error::Output(e), now))?;
        Ok(true)
    }
    fn fail(&mut self, error: Error, now: u64) -> Error {
        let error = *self.fault.get_or_insert(error);
        self.wanted = false;
        let _ = self.port.shutdown(now);
        error
    }
}
