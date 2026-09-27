use crate::{Action, Error, Frame, Id, RemoteError, Request, Response, State};
use core::fmt;
use stagemaster_install::{Commit, Identity, MAX_CHUNK_BYTES, Phase, Transaction};
use stagemaster_package::{Archive, ReadAt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Installed(Commit),
    Cancelled,
    NotStarted,
}

#[derive(Debug)]
pub enum UploadError {
    Wire(Error),
    Remote(RemoteError),
    Source(stagemaster_package::Error),
}
impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wire(e) => e.fmt(f),
            Self::Remote(e) => e.fmt(f),
            Self::Source(e) => e.fmt(f),
        }
    }
}
impl core::error::Error for UploadError {}
impl From<Error> for UploadError {
    fn from(e: Error) -> Self {
        Self::Wire(e)
    }
}
impl From<RemoteError> for UploadError {
    fn from(e: RemoteError) -> Self {
        Self::Remote(e)
    }
}
impl From<stagemaster_package::Error> for UploadError {
    fn from(e: stagemaster_package::Error) -> Self {
        Self::Source(e)
    }
}

/// One explicit install intent over replaceable connections. Source must remain immutable.
/// No timers or I/O: adapters transmit `outbound()`, deliver `accept()`, and reconnect after partial loss.
pub struct Upload<R> {
    source: R,
    identity: Identity,
    link: Option<Id>,
    last_link: Option<Id>,
    next_id: Option<u64>,
    pending: Option<Frame>,
    latest: Option<State>,
    outcome: Option<Outcome>,
    cancel: bool,
    halted: Option<RemoteError>,
    query: bool,
}
enum Next {
    Action(CommandAction),
    Finish(Outcome),
}
enum CommandAction {
    Status,
    Begin(Transaction),
    Write(Transaction, usize, usize),
    Verify(Transaction),
    Commit(Transaction),
    Cancel(Transaction),
    Reconcile(Transaction),
}
impl<R: ReadAt> Upload<R> {
    /// Fully validate before requesting device mutation; never buffer the entire source.
    /// # Errors
    /// Reject invalid, incompatible or oversized source packages.
    pub fn new(source: R) -> Result<Self, stagemaster_package::Error> {
        let identity = Identity::from_archive(&Archive::open(&source)?);
        Ok(Self {
            source,
            identity,
            link: None,
            last_link: None,
            next_id: Some(1),
            pending: None,
            latest: None,
            outcome: None,
            cancel: false,
            halted: None,
            query: true,
        })
    }
    /// Called with a new connection identity obtained via the trusted connection adapter.
    /// # Errors
    /// Reject a zero or immediately reused identity; reconnect must really create a new link.
    pub fn connect(&mut self, link: Id) -> Result<(), Error> {
        if link == [0; 16] || self.last_link == Some(link) {
            return Err(Error::Connection);
        }
        self.link = Some(link);
        self.last_link = Some(link);
        self.next_id = Some(1);
        self.pending = None;
        self.latest = None;
        self.halted = None;
        self.query = true;
        Ok(())
    }
    pub fn disconnect(&mut self) {
        self.link = None;
        self.pending = None;
        self.latest = None;
        self.query = true;
    }
    #[must_use]
    pub const fn state(&self) -> Option<State> {
        self.latest
    }
    #[must_use]
    pub const fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }
    #[must_use]
    pub const fn identity(&self) -> Identity {
        self.identity
    }
    /// Cancel intent waits for an in-flight response or a reconnect/status query.
    pub fn request_cancel(&mut self) {
        self.cancel = true;
        self.halted = None;
        self.query = true;
    }
    /// Explicit retry queries authoritative state; it never discards a failed device transaction.
    /// # Errors
    /// An outstanding request must first be answered or abandoned with its connection.
    pub fn retry(&mut self) -> Result<(), Error> {
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        self.halted = None;
        self.query = true;
        Ok(())
    }
    /// Return the identical frame until accept succeeds; adapters must not append it to a half-frame.
    /// # Errors
    /// Preserve peer errors, exhausted counters, connection failures and source read errors.
    pub fn outbound(&mut self) -> Result<Option<&Frame>, UploadError> {
        if self.pending.is_some() {
            return Ok(self.pending.as_ref());
        }
        if self.outcome.is_some() {
            return Ok(None);
        }
        if let Some(e) = self.halted {
            return Err(e.into());
        }
        let link = self.link.ok_or(Error::Connection)?;
        let next = if self.query || self.latest.is_none() {
            Next::Action(CommandAction::Status)
        } else {
            self.decide(self.latest.ok_or(Error::State)?)?
        };
        let Next::Action(action) = next else {
            if let Next::Finish(outcome) = next {
                self.outcome = Some(outcome);
            }
            return Ok(None);
        };
        let id = self.next_id.ok_or(Error::Exhausted)?;
        let mut buffer = [0; MAX_CHUNK_BYTES];
        let action = match action {
            CommandAction::Status => Action::Status,
            CommandAction::Begin(transaction) => Action::Begin {
                transaction,
                identity: self.identity,
            },
            CommandAction::Write(transaction, offset, count) => {
                self.source.read_exact(offset, &mut buffer[..count])?;
                Action::Write {
                    transaction,
                    offset,
                    bytes: &buffer[..count],
                }
            }
            CommandAction::Verify(t) => Action::Verify(t),
            CommandAction::Commit(t) => Action::Commit(t),
            CommandAction::Cancel(t) => Action::Cancel(t),
            CommandAction::Reconcile(t) => Action::Reconcile(t),
        };
        self.pending = Some(Request { link, id, action }.encode()?);
        Ok(self.pending.as_ref())
    }
    /// Accept only a matching application response, not an ATT write acknowledgement.
    /// # Errors
    /// Stale/malformed responses leave the in-flight request intact. Remote failures halt retries.
    pub fn accept(&mut self, bytes: &[u8]) -> Result<(), UploadError> {
        let response = Response::decode(bytes)?;
        let request = Request::decode(self.pending.as_ref().ok_or(Error::State)?.bytes())?;
        if response.link != request.link
            || response.id != request.id
            || response.command != request.action.command()
        {
            return Err(Error::Connection.into());
        }
        if response.result.is_ok() {
            self.validate_success(request.action, response.state)?;
        }
        self.next_id = request.id.checked_add(1);
        self.pending = None;
        self.latest = Some(response.state);
        self.query = false;
        match response.result {
            Ok(()) | Err(RemoteError::Uncertain) => Ok(()),
            Err(error) => {
                self.halted = Some(error);
                Err(error.into())
            }
        }
    }
    fn validate_success(&self, action: Action<'_>, state: State) -> Result<(), Error> {
        if action == Action::Status {
            return Ok(());
        }
        let transaction = action.transaction().ok_or(Error::State)?;
        let p = state.progress.ok_or(Error::State)?;
        if p.transaction != transaction || p.identity != self.identity || !state.owned {
            return Err(Error::State);
        }
        let valid = match action {
            Action::Begin { identity, .. } => p.identity == identity,
            Action::Write { offset, bytes, .. } => {
                p.phase == Phase::Receiving && p.received == offset + bytes.len()
            }
            Action::Verify(_) => matches!(p.phase, Phase::Verified | Phase::Committed),
            Action::Commit(_) => p.phase == Phase::Committed && state.head == Some(p.commit),
            Action::Cancel(_) => p.phase == Phase::Cancelled,
            Action::Reconcile(_) => matches!(p.phase, Phase::Committed | Phase::Failed),
            Action::Status => true,
        };
        if !valid {
            return Err(Error::State);
        }
        Ok(())
    }
    fn decide(&self, state: State) -> Result<Next, RemoteError> {
        if let Some(head) = state.head.filter(|c| c.identity == self.identity) {
            return Ok(Next::Finish(Outcome::Installed(head)));
        }
        if self.identity.bytes > state.max_package {
            return Err(RemoteError::Bounds);
        }
        if let Some(p) = state.progress {
            let terminal = matches!(p.phase, Phase::Committed | Phase::Cancelled);
            if !terminal {
                if !state.owned {
                    return Err(RemoteError::Ownership);
                }
                if p.identity != self.identity {
                    return Err(RemoteError::Conflict);
                }
                let action = if p.phase == Phase::Uncertain {
                    CommandAction::Reconcile(p.transaction)
                } else if self.cancel {
                    CommandAction::Cancel(p.transaction)
                } else {
                    match p.phase {
                        Phase::Receiving if p.received < p.identity.bytes => CommandAction::Write(
                            p.transaction,
                            p.received,
                            state.max_chunk.min(p.identity.bytes - p.received),
                        ),
                        Phase::Receiving => CommandAction::Verify(p.transaction),
                        Phase::Verified => CommandAction::Commit(p.transaction),
                        _ => return Err(RemoteError::State),
                    }
                };
                return Ok(Next::Action(action));
            }
            if self.cancel {
                return Ok(Next::Finish(
                    if p.identity == self.identity && state.owned && p.phase == Phase::Cancelled {
                        Outcome::Cancelled
                    } else {
                        Outcome::NotStarted
                    },
                ));
            }
        } else if self.cancel {
            return Ok(Next::Finish(Outcome::NotStarted));
        }
        let counter = state
            .progress
            .map_or(0, |p| p.transaction.counter)
            .checked_add(1)
            .ok_or(RemoteError::Exhausted)?;
        Ok(Next::Action(CommandAction::Begin(Transaction {
            boot: state.boot,
            counter,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use stagemaster_install::{Progress, Slot};

    #[test]
    fn final_request_success_is_reported_before_sequence_exhaustion() {
        let identity = Identity {
            bytes: 64,
            digest: [3; 32],
        };
        let transaction = Transaction {
            boot: [2; 16],
            counter: 1,
        };
        let commit = Commit {
            slot: Slot::A,
            generation: 1,
            identity,
        };
        // Seed the finite-state boundary directly: iterating 2^64 requests cannot test exhaustion.
        // This path must not read the source; it consumes only the matching final commit receipt.
        let mut upload = Upload {
            source: &[][..],
            identity,
            link: Some([1; 16]),
            last_link: Some([1; 16]),
            next_id: Some(u64::MAX),
            pending: Some(
                Request {
                    link: [1; 16],
                    id: u64::MAX,
                    action: Action::Commit(transaction),
                }
                .encode()
                .unwrap(),
            ),
            latest: None,
            outcome: None,
            cancel: false,
            halted: None,
            query: false,
        };
        let response = Response {
            link: [1; 16],
            id: u64::MAX,
            command: Command::Commit,
            result: Ok(()),
            state: State {
                boot: transaction.boot,
                head: Some(commit),
                progress: Some(Progress {
                    transaction,
                    identity,
                    received: 64,
                    phase: Phase::Committed,
                    commit,
                }),
                owned: true,
                max_chunk: 1024,
                max_package: 2 * 1024 * 1024,
            },
        };
        upload.accept(response.encode().unwrap().bytes()).unwrap();
        assert_eq!(upload.next_id, None);
        assert!(upload.outbound().unwrap().is_none());
        assert_eq!(upload.outcome(), Some(Outcome::Installed(commit)));
    }
}
