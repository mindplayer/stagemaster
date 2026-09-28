//! Native unbonded transport; device key trust comes only from injected local credentials.
mod handshake;
mod incoming;
mod io;
use super::{C, Characteristic, Peripheral, Problem};
use incoming::Incoming;
use stagemaster_device_link::{management::ApplicationReceipt, secure::Sender};
use stagemaster_device_session::Channel as Secure;
use tokio::time::Instant;
use uuid::Uuid;
pub(super) const SERVICE: Uuid = Uuid::from_u128(0xf889eda0_0100_4e83_968e_799ab99558fa);
const REQUEST: Uuid = Uuid::from_u128(0xf889eda2_0100_4e83_968e_799ab99558fa);
const RESPONSE: Uuid = Uuid::from_u128(0xf889eda3_0100_4e83_968e_799ab99558fa);
fn wire(error: impl core::fmt::Display) -> Problem {
    Problem::new(C::Installation).detail(error.to_string())
}
fn now(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}
pub(super) struct Channel {
    receipt: ApplicationReceipt,
    secure: Secure,
    sender: Sender,
    incoming: Incoming,
    request: Characteristic,
    origin: Instant,
    received_at: u64,
    until: u64,
    pending: Option<Vec<u8>>,
    usable: bool,
}
impl Channel {
    pub fn peer(&self) -> Option<crate::InstallationPeer> {
        let now = now(self.origin);
        (self.usable
            && self.incoming.healthy()
            && now < self.until
            && now.saturating_sub(self.received_at) < stagemaster_device_session::LEASE_MS)
            .then_some(crate::InstallationPeer {
                device: self.receipt.device,
                boot: self.receipt.boot,
                session: self.receipt.session,
                authentication: stagemaster_device_session::AUTHENTICATION,
                fragment_bytes: stagemaster_device_link::management::MESSAGE_BYTES,
                message_bytes: stagemaster_device_link::management::MESSAGE_BYTES,
            })
    }
    pub fn correlate(&self, bytes: &[u8]) -> Result<(), Problem> {
        let desc = stagemaster_device_info::Description::decode(bytes).map_err(wire)?;
        if desc.device != self.receipt.device
            || desc.boot != self.receipt.boot
            || desc.session != self.receipt.diagnostic
            || desc.authentication != stagemaster_device_session::AUTHENTICATION
        {
            return Err(Problem::new(C::Installation));
        }
        Ok(())
    }
    fn check(&mut self) -> Result<(), Problem> {
        if self.peer().is_none() {
            self.usable = false;
            self.secure.close();
            return Err(Problem::new(C::Installation));
        }
        self.secure.poll(now(self.origin)).map_err(wire)
    }
    fn checked<T>(&mut self, result: Result<T, Problem>) -> Result<T, Problem> {
        if result.is_err() {
            self.usable = false;
            self.secure.close();
            self.pending = None;
        }
        result
    }
}
