//! Internal bounded credential queue to the sole Flash executor. Never a GATT API.
mod keys;
pub mod link;
mod startup;
mod store;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, with_timeout};
#[cfg(feature = "binding-local-test")]
use stagemaster_device_auth::LocalIdentity;
use stagemaster_device_auth::Vault;
pub use store::Store;

// One fixed-capacity cross-core queue; keep secret ownership inline and avoid
// fallible heap allocation when revoking/replacing credentials. Vault is bounded.
#[allow(clippy::large_enum_variant)]
pub enum Action {
    Recover,
    #[cfg(feature = "binding-local-test")]
    Initialize(LocalIdentity),
    Commit(Vault),
}

pub fn random() -> [u8; 16] {
    loop {
        let mut bytes = [0; 16];
        esp_hal::rng::Rng::new().read(&mut bytes);
        if bytes != [0; 16] {
            return bytes;
        }
    }
}
pub struct Request {
    id: u32,
    action: Action,
}
struct Reply {
    id: u32,
    result: Result<Vault, Error>,
}
#[derive(Clone, Copy, Debug)]
pub enum Error {
    Busy,
    Deadline,
    Unavailable,
    Storage,
    Maintenance,
    Exhausted,
}
static REQUESTS: Channel<CriticalSectionRawMutex, Request, 1> = Channel::new();
static REPLIES: Channel<CriticalSectionRawMutex, Reply, 1> = Channel::new();

/// Keep one client for the entire radio task, including reconnects.
pub struct Client {
    next: u32,
}
impl Client {
    pub const fn new() -> Self {
        Self { next: 0 }
    }
    pub async fn call(&mut self, action: Action) -> Result<Vault, Error> {
        self.next = self.next.checked_add(1).ok_or(Error::Exhausted)?;
        let id = self.next;
        REQUESTS
            .try_send(Request { id, action })
            .map_err(|_| Error::Busy)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(Error::Deadline)?;
            let reply = with_timeout(remaining, REPLIES.receive())
                .await
                .map_err(|_| Error::Deadline)?;
            if reply.id == id {
                return reply.result;
            }
            if reply.id > id {
                return Err(Error::Unavailable);
            }
            // Drain old results after cancellation/timeout. Never grant to an old connection.
        }
    }
}
