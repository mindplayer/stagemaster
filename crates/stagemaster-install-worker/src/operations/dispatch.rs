use super::{Connection, Detail, Failure, Operation, Program, Step, Text};
use crate::ManagedWorker;
use stagemaster_install::Storage;
use stagemaster_runtime::{Code, Grant, MaintenanceError, Origin, PlaybackPolicy, Request};

impl From<Code> for Failure {
    fn from(value: Code) -> Self {
        Self::Runtime(value)
    }
}
impl Connection {
    pub(super) fn apply<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        operation: Operation,
        now: u64,
    ) -> Result<Detail, Failure> {
        match operation {
            Operation::Status => {}
            Operation::Catalog { index } => return program(device, index),
            Operation::Step { index } => return step(device, index),
            Operation::Acquire {
                duration_ms,
                takeover,
            } => {
                self.duration(duration_ms, now)?;
                self.lease = Some(device.acquire(
                    Grant {
                        principal: self.grant.principal(),
                        origin: Origin::Remote,
                        duration_ms,
                    },
                    takeover,
                    now,
                )?);
            }
            Operation::Renew { duration_ms } => {
                self.duration(duration_ms, now)?;
                device.renew(self.lease.ok_or(Code::Lease)?, duration_ms, now)?;
            }
            Operation::Release => {
                device.release(self.lease.ok_or(Code::Lease)?, now)?;
                self.lease = None;
            }
            Operation::Apply(action) => {
                let lease = self.lease.ok_or(Code::Lease)?;
                let state = device.state();
                let owner = state
                    .owner
                    .filter(|o| o.lease == lease)
                    .ok_or(Code::Lease)?;
                device
                    .submit(
                        Request {
                            lease,
                            serial: owner.serial.checked_add(1).ok_or(Code::Exhausted)?,
                            expected_revision: state.revision,
                            action,
                        },
                        now,
                    )?
                    .result?;
            }
            Operation::FinishMaintenance => {
                let lease = self.lease.ok_or(Code::Lease)?;
                if device
                    .state()
                    .owner
                    .is_none_or(|owner| owner.lease != lease)
                {
                    return Err(Code::Lease.into());
                }
                device
                    .finish_maintenance(now)
                    .map_err(|error| match error {
                        MaintenanceError::State(code) => Failure::Runtime(code),
                        MaintenanceError::Load(_) => Failure::Storage,
                    })?;
            }
        }
        Ok(Detail::State)
    }

    fn duration(&self, duration: u64, now: u64) -> Result<(), Code> {
        if !(1..=stagemaster_runtime::MAX_LEASE_MS).contains(&duration)
            || duration > self.grant.expires_at().saturating_sub(now)
        {
            Err(Code::Identity)
        } else {
            Ok(())
        }
    }
}

fn program<S: Storage, P: PlaybackPolicy>(
    device: &ManagedWorker<S, P>,
    index: u16,
) -> Result<Detail, Failure> {
    let entries = device.catalog();
    let index = usize::from(index);
    if index > entries.len() {
        return Err(Code::Selection.into());
    }
    let entry = entries
        .get(index)
        .map(|entry| -> Result<Program, Failure> {
            Ok(Program {
                key: stagemaster_runtime::ProgramKey {
                    kind: entry.kind,
                    id: entry.id,
                },
                name: Text::copy(&entry.name)?,
                loader_bytes: u32::try_from(entry.usage.loader_peak_bytes)
                    .map_err(|_| Failure::Bounds)?,
            })
        })
        .transpose()?;
    Ok(Detail::Program(entry))
}
fn step<S: Storage, P: PlaybackPolicy>(
    device: &ManagedWorker<S, P>,
    index: u16,
) -> Result<Detail, Failure> {
    let entries = device.steps();
    let index = usize::from(index);
    if index > entries.len() {
        return Err(Code::Step.into());
    }
    let entry = entries
        .get(index)
        .map(|entry| -> Result<Step, Failure> {
            Ok(Step {
                id: entry.id,
                name: Text::copy(&entry.name)?,
                number: Text::copy(&entry.number)?,
            })
        })
        .transpose()?;
    Ok(Detail::Step(entry))
}
