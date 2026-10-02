use crate::Error;
use stagemaster_runtime::{Code, FrameInfo, State};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Running,
    Stopping,
    Stopped,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Runtime(Code),
    Panic,
}

/// A software sample, never an acknowledgement of physical transmission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub info: FrameInfo,
    pub slots: [u8; 512],
}
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub state: State,
    pub frame: Option<Frame>,
    pub cycles: u64,
    pub skipped_publications: u64,
    pub max_lateness_ms: u64,
    pub missed_periods: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub phase: Phase,
    pub fault: Option<Fault>,
    /// Terminal/stopping hosts never expose a still-valid frame or runtime snapshot.
    pub snapshot: Option<Snapshot>,
}
#[derive(Debug)]
pub(crate) struct Shared {
    pub stop: AtomicBool,
    phase: AtomicU8,
    fault: OnceLock<Fault>,
    pub snapshot: Mutex<Snapshot>,
}
impl Shared {
    pub fn new(state: State) -> Self {
        Self {
            stop: AtomicBool::new(false),
            phase: AtomicU8::new(Phase::Running as u8),
            fault: OnceLock::new(),
            snapshot: Mutex::new(Snapshot {
                state,
                frame: None,
                cycles: 0,
                skipped_publications: 0,
                max_lateness_ms: 0,
                missed_periods: 0,
            }),
        }
    }
    pub fn phase(&self) -> Phase {
        match self.phase.load(Ordering::Acquire) {
            0 => Phase::Running,
            1 => Phase::Stopping,
            2 => Phase::Stopped,
            _ => Phase::Faulted,
        }
    }
    pub fn finish(&self, fault: Option<Fault>) {
        if let Some(reason) = fault {
            let _ = self.fault.set(reason);
        }
        self.phase.store(
            if fault.is_some() {
                Phase::Faulted
            } else {
                Phase::Stopped
            } as u8,
            Ordering::Release,
        );
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
        // Do not overwrite an already terminal failure with Stopping.
        let _ = self.phase.compare_exchange(
            Phase::Running as u8,
            Phase::Stopping as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}
#[derive(Clone, Debug)]
pub struct Observer {
    pub(crate) shared: Arc<Shared>,
}
impl Observer {
    #[must_use]
    pub fn phase(&self) -> Phase {
        self.shared.phase()
    }

    /// Return copied state only; consumers cannot hold the publisher's lock.
    /// # Errors
    /// A contended or poisoned observation slot reports Busy; it never delays the runtime.
    pub fn read(&self) -> Result<Observation, Error> {
        let phase = self.phase();
        if phase != Phase::Running {
            return Ok(Observation {
                phase,
                fault: self.fault(phase),
                snapshot: None,
            });
        }
        let snapshot = *self
            .shared
            .snapshot
            .try_lock()
            .map_err(|_| Error::ObservationBusy)?;
        let phase = self.phase();
        Ok(Observation {
            phase,
            fault: self.fault(phase),
            snapshot: (phase == Phase::Running).then_some(snapshot),
        })
    }
    fn fault(&self, phase: Phase) -> Option<Fault> {
        if phase == Phase::Faulted {
            self.shared.fault.get().copied()
        } else {
            None
        }
    }
}
