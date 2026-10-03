use super::{Command, Completion, Live, Work};
use crate::{Epoch, ManagedWorker, operations::Connection};
use stagemaster_device_auth::application::Grant;
use stagemaster_install::Storage;
use stagemaster_runtime::{Code, PlaybackPolicy};
use stagemaster_runtime_protocol::Frame;

type Error = crate::operations::Error;

/// Keep one endpoint on the original worker, across wireless connections.
/// The adapter must schedule tick independently and never block it sending completions.
#[derive(Default)]
pub struct Endpoint {
    active: Option<(Epoch, Connection)>,
    last_opened: u32,
}
impl Endpoint {
    /// Advance autonomous execution and clean up revoked input even with no requests.
    /// # Errors
    /// Runtime/clock failures are preserved. Expired input alone is not a playback error.
    pub fn tick<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        now: u64,
        mut live: impl FnMut() -> Option<Live>,
    ) -> Result<(), Code> {
        let result = if let Some((epoch, connection)) = &mut self.active {
            connection.poll(device, now, |t| current(&mut live, *epoch, t))
        } else {
            Ok(())
        };
        if result.is_err() {
            // poll already relinquished this connection's lease.
            self.active = None;
        }
        if let Err(Error::Runtime(code)) = result {
            return Err(code);
        }
        device.tick(now)
    }

    /// Execute on the storage/runtime owner; never in the radio callback. Reread the
    /// independently synchronized Live slot at every check, including after slow I/O.
    /// Already executed effects are not rolled back when their completion is obsolete.
    pub fn process<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        command: Command,
        mut clock: impl FnMut() -> u64,
        mut live: impl FnMut() -> Option<Live>,
    ) -> Completion {
        let result = self.execute(device, &command, &mut clock, &mut live);
        Completion {
            epoch: command.epoch,
            ticket: command.ticket,
            result,
        }
    }

    fn execute<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        command: &Command,
        clock: &mut impl FnMut() -> u64,
        live: &mut impl FnMut() -> Option<Live>,
    ) -> Result<Frame, Error> {
        let now = clock();
        self.tick(device, now, &mut *live).map_err(Error::Runtime)?;
        let epoch = command.epoch;
        if current(live, epoch, now).is_none() || now >= command.deadline {
            return Err(Error::Obsolete);
        }
        let mut access = |time| {
            if time < command.deadline {
                current(live, epoch, time)
            } else {
                None
            }
        };
        let result = match command.work {
            Work::Open(offer) => {
                if self.active.is_some() || epoch.get() <= self.last_opened {
                    return Err(Error::Obsolete);
                }
                self.last_opened = epoch.get();
                let mut connection = Connection::open(device, now, &mut access)?;
                let frame = offer.encode().map_err(Error::Protocol)?;
                let result = connection.negotiate(device, frame.bytes(), now, &mut access);
                self.active = Some((epoch, connection));
                result
            }
            Work::Request(request) => {
                let (_, connection) = self
                    .active
                    .as_mut()
                    .filter(|(active, _)| *active == epoch)
                    .ok_or(Error::Obsolete)?;
                let frame = request.encode().map_err(Error::Protocol)?;
                connection.process_message(device, frame.bytes(), &mut *clock, &mut access)
            }
        };
        // Includes negotiation, which also touches the original runtime.
        let now = clock();
        self.tick(device, now, &mut *live).map_err(Error::Runtime)?;
        let still_live = now < command.deadline && current(live, epoch, now).is_some();
        if (!still_live || (result.is_err() && !matches!(result, Err(Error::Identity))))
            && let Some((_, mut connection)) = self.active.take()
        {
            connection.close(device).map_err(Error::Runtime)?;
        }
        if !still_live {
            return Err(Error::Obsolete);
        }
        result
    }
}

fn current(live: &mut impl FnMut() -> Option<Live>, epoch: Epoch, now: u64) -> Option<Grant> {
    live()?.filter_epoch(epoch, now)
}
impl Live {
    fn filter_epoch(&self, epoch: Epoch, now: u64) -> Option<Grant> {
        (self.epoch == epoch).then(|| self.grant(now)).flatten()
    }
}
