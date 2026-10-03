use super::{Connection, Error};
use crate::ManagedWorker;
use stagemaster_device_auth::application::Grant;
use stagemaster_install::Storage;
use stagemaster_runtime::{Code, PlaybackPolicy};

impl Connection {
    /// Call even without incoming commands. Runtime scheduling remains independently owned.
    /// # Errors
    /// Invalid time, access or boot closes only this connection and relinquishes its input.
    pub fn poll<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        now: u64,
        mut live: impl FnMut(u64) -> Option<Grant>,
    ) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let result = if now < self.last_ms || now < device.state().observed_ms {
            Err(Error::Clock)
        } else if device.state().boot != self.grant.context().boot {
            Err(Error::Identity)
        } else if now >= self.grant.expires_at() || live(now) != Some(self.grant) {
            Err(Error::Obsolete)
        } else {
            self.last_ms = now;
            device.tick(now).map_err(Error::Runtime)
        };
        if result.is_err() {
            self.close(device).map_err(Error::Runtime)?;
        }
        result
    }

    /// Disconnect is never Stop. A previous connection cannot release a new owner.
    /// Call on cancellation/teardown; Drop cannot borrow the independently owned runtime.
    /// If omitted, the bounded Runtime lease still expires; no new requests are possible
    /// after the caller has discarded this connection.
    /// # Errors
    /// Preserve a core failure while relinquishing this connection's input authority.
    pub fn close<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
    ) -> Result<(), Code> {
        self.closed = true;
        self.wire_ready = false;
        self.last = None;
        if let Some(lease) = self.lease.take() {
            let state = device.state();
            if state.owner.is_some_and(|owner| owner.lease == lease) {
                // Cleanup uses the last trusted device time, even after a caller clock fault.
                device.release(lease, state.observed_ms)?;
            }
        }
        Ok(())
    }
}
