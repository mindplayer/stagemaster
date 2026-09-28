//! Serialized install ownership, separate from radio callbacks and authentication.
//! The adapter's live epoch is a revocation channel independent of its bounded queue.
#![no_std]
#![forbid(unsafe_code)]

mod endpoint;
mod managed;
#[cfg(feature = "application")]
pub mod secure;
pub use endpoint::{ChannelError, Endpoint, Phase};
pub use managed::ManagedWorker;

use core::num::NonZeroU32;
use stagemaster_install::{Installed, Installer, Storage};
use stagemaster_transfer::{AuthorizedLink, Frame, Service};

/// Adapter-issued connection generation. Never decode it from a peer request.
pub type Epoch = NonZeroU32;

/// Trusted internal queue. This type intentionally has no wire decoder.
// Fixed-capacity queues reserve the maximum frame up front, without an allocator.
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Open { epoch: Epoch, link: AuthorizedLink },
    Frame { epoch: Epoch, frame: Frame },
}
impl Command {
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        match self {
            Self::Open { epoch, .. } | Self::Frame { epoch, .. } => *epoch,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Obsolete,
    NotOpen,
    Protocol(stagemaster_transfer::Error),
    Maintenance(stagemaster_runtime::Code),
}
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // Same bounded, allocation-free queue as Command.
pub enum Reply {
    Opened,
    Frame(Frame),
}
#[derive(Debug)]
pub struct Completion {
    pub epoch: Epoch,
    pub result: Result<Reply, Error>,
}

/// Create and keep this object on its single worker. The store need not be Send.
pub struct Worker<S> {
    service: Service<S>,
    active: Option<Epoch>,
    last_opened: u32,
}
impl<S: Storage> Worker<S> {
    /// # Errors
    /// The installer must not already have an unowned volatile transaction.
    pub fn new(installer: Installer<S>) -> Result<Self, stagemaster_transfer::Error> {
        Ok(Self {
            service: Service::new(installer)?,
            active: None,
            last_opened: 0,
        })
    }

    /// Observe independent revocation while idle, or before dequeuing work.
    /// Detach retains transaction ownership and any durable result for reconciliation.
    pub fn observe(&mut self, live: Option<Epoch>) {
        if self.active != live {
            self.service.detach();
            self.active = None;
        }
    }

    /// Execute one queued command, checking a trusted live epoch before and after I/O.
    /// An already started operation may complete after revocation; no obsolete reply
    /// may be presented as the result of a later connection. Recheck at notification too.
    pub fn process(
        &mut self,
        command: Command,
        mut live: impl FnMut() -> Option<Epoch>,
    ) -> Completion {
        let epoch = command.epoch();
        let current = live();
        self.observe(current);
        let result = if current == Some(epoch) {
            match command {
                Command::Open { link, .. } => {
                    if epoch.get() <= self.last_opened || self.active.is_some() {
                        Err(Error::Obsolete)
                    } else {
                        // Consume even failed opens. A late/retried command cannot revive one.
                        self.last_opened = epoch.get();
                        self.service
                            .attach(link)
                            .map_err(Error::Protocol)
                            .map(|()| {
                                self.active = Some(epoch);
                                Reply::Opened
                            })
                    }
                }
                Command::Frame { frame, .. } => {
                    if self.active == Some(epoch) {
                        match self.service.process(frame.bytes()) {
                            Ok(frame) => Ok(Reply::Frame(frame)),
                            Err(error) => {
                                self.active = None;
                                Err(Error::Protocol(error))
                            }
                        }
                    } else {
                        Err(Error::NotOpen)
                    }
                }
            }
        } else {
            Err(Error::Obsolete)
        };
        let current = live();
        self.observe(current);
        Completion {
            epoch,
            result: if current == Some(epoch) {
                result
            } else {
                Err(Error::Obsolete)
            },
        }
    }

    /// Immutable installed data for a runtime living on this same worker.
    /// # Errors
    /// Preserve missing installation, storage and package validation errors.
    pub fn snapshot(&self) -> Result<Installed<S::Snapshot>, stagemaster_install::Error<S::Error>> {
        self.service.snapshot()
    }
}
