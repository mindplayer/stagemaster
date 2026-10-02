use crate::{Action, Error, MAX_COMMAND_TTL, Observer, Phase, observation::Shared};
use stagemaster_runtime::{Grant, Lease, Receipt, Request, State};
use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    time::{Duration, Instant},
};

pub(crate) type Reply<T> = SyncSender<Result<T, Error>>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Acquisition {
    pub lease: Lease,
    pub state: State,
}
pub(crate) enum Command {
    Acquire {
        grant: Grant,
        takeover: bool,
        reply: Reply<Acquisition>,
    },
    Submit {
        request: Request,
        reply: Reply<Receipt>,
    },
    Renew {
        lease: Lease,
        duration_ms: u64,
        reply: Reply<()>,
    },
    Release {
        lease: Lease,
        reply: Reply<()>,
    },
}
pub(crate) struct Envelope {
    pub deadline: Instant,
    pub command: Command,
}
#[derive(Clone, Debug)]
pub(crate) struct Ingress {
    pub sender: SyncSender<Envelope>,
    pub shared: Arc<Shared>,
}
impl Ingress {
    pub fn send<T>(
        &self,
        ttl: Duration,
        make: impl FnOnce(Reply<T>) -> Command,
    ) -> Result<Ticket<T>, Error> {
        if ttl.is_zero() || ttl > MAX_COMMAND_TTL {
            return Err(Error::InvalidDeadline);
        }
        if self.shared.phase() != Phase::Running {
            return Err(Error::Closed);
        }
        let deadline = Instant::now()
            .checked_add(ttl)
            .ok_or(Error::InvalidDeadline)?;
        let (reply, receiver) = mpsc::sync_channel(1);
        self.sender
            .try_send(Envelope {
                deadline,
                command: make(reply),
            })
            .map_err(|e| match e {
                mpsc::TrySendError::Full(_) => Error::QueueFull,
                mpsc::TrySendError::Disconnected(_) => Error::Closed,
            })?;
        Ok(Ticket { receiver })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitError {
    /// Waiting has ended, but the accepted operation has NOT been cancelled.
    Timeout,
    /// No receipt is available. Do not infer that the operation was never applied.
    Unavailable,
}
impl std::fmt::Display for WaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Timeout => "等待超时，操作未取消，可继续查询原回执",
            Self::Unavailable => "无法取得操作回执，执行结果须重新核对",
        })
    }
}
impl std::error::Error for WaitError {}

#[derive(Debug)]
pub struct Ticket<T> {
    receiver: Receiver<Result<T, Error>>,
}
impl<T> Ticket<T> {
    /// Consume the one result, keeping the ticket usable after a timeout.
    /// # Errors
    /// Timeout is not cancellation; Unavailable is not proof of non-execution.
    pub fn wait(&self, timeout: Duration) -> Result<Result<T, Error>, WaitError> {
        self.receiver.recv_timeout(timeout).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => WaitError::Timeout,
            mpsc::RecvTimeoutError::Disconnected => WaitError::Unavailable,
        })
    }
}

#[derive(Debug)]
pub struct Connection {
    pub(crate) ticket: Ticket<Acquisition>,
    pub(crate) ingress: Ingress,
}
impl Connection {
    /// Construct a scoped client only from the actual acquired lease.
    /// # Errors
    /// The same wait/outcome rules apply as for command tickets.
    pub fn wait(&self, timeout: Duration) -> Result<Result<Client, Error>, WaitError> {
        self.ticket.wait(timeout).map(|r| {
            r.map(|acquired| Client {
                lease: acquired.lease,
                acquisition: acquired.state,
                ingress: self.ingress.clone(),
            })
        })
    }
}

/// Dropping a client never stops playback. Input ownership ends by release or lease expiry.
#[derive(Clone, Debug)]
pub struct Client {
    lease: Lease,
    acquisition: State,
    ingress: Ingress,
}
impl Client {
    /// Historical acquisition result, independent of delayed observer publication.
    /// Use its revision for the first command; later commands use their own receipts.
    #[must_use]
    pub const fn acquired_state(&self) -> State {
        self.acquisition
    }
    #[must_use]
    pub fn observer(&self) -> Observer {
        Observer {
            shared: self.ingress.shared.clone(),
        }
    }

    /// Preserve serial/revision/payload when retrying one logical operation.
    /// # Errors
    /// Queue/deadline/lifecycle rejection means no admission; business outcome is in the ticket.
    pub fn submit(
        &self,
        serial: u64,
        expected_revision: u64,
        action: Action,
        ttl: Duration,
    ) -> Result<Ticket<Receipt>, Error> {
        let request = Request {
            lease: self.lease,
            serial,
            expected_revision,
            action: action.into(),
        };
        self.ingress
            .send(ttl, |reply| Command::Submit { request, reply })
    }
    /// # Errors
    /// Admission can fail; lease validation occurs on the runtime thread.
    pub fn renew(&self, duration_ms: u64, ttl: Duration) -> Result<Ticket<()>, Error> {
        self.ingress.send(ttl, |reply| Command::Renew {
            lease: self.lease,
            duration_ms,
            reply,
        })
    }
    /// # Errors
    /// An expired or replaced lease cannot release a newer controller.
    pub fn release(&self, ttl: Duration) -> Result<Ticket<()>, Error> {
        self.ingress.send(ttl, |reply| Command::Release {
            lease: self.lease,
            reply,
        })
    }
}
