use crate::{Action, Error, Frame, Id, RemoteError, Request, Response, State};
use sha2::{Digest, Sha256};
use stagemaster_install::{Code, Installed, Installer, MAX_CHUNK_BYTES, Phase, Storage};
use stagemaster_package::MAX_PACKAGE_BYTES;

/// Trusted host input, never decoded from peer bytes. Not an authentication implementation.
/// The host must verify device-scoped install permission and generate a fresh random session.
#[derive(Clone, Copy, Debug)]
pub struct AuthorizedLink {
    pub principal: Id,
    pub session: Id,
}
struct Link {
    principal: Id,
    session: Id,
    last_id: u64,
    digest: [u8; 32],
    response: Option<Frame>,
}
pub struct Service<S> {
    installer: Installer<S>,
    owner: Option<Id>,
    link: Option<Link>,
    last_link: Option<Id>,
}
impl<S: Storage> Service<S> {
    /// # Errors
    /// Existing in-memory transactions have unknown ownership; create directly after open.
    pub fn new(installer: Installer<S>) -> Result<Self, Error> {
        if installer.progress().is_some() {
            return Err(Error::State);
        }
        Ok(Self {
            installer,
            owner: None,
            link: None,
            last_link: None,
        })
    }
    /// Install an explicitly authorized connection. Never call from a diagnostic handshake.
    /// # Errors
    /// Refuse missing identities, concurrent links and reuse of the last link identity.
    pub fn attach(&mut self, authorization: AuthorizedLink) -> Result<(), Error> {
        if authorization.principal == [0; 16] || authorization.session == [0; 16] {
            return Err(Error::Denied);
        }
        if self.link.is_some() {
            return Err(Error::Busy);
        }
        if self.last_link == Some(authorization.session) {
            return Err(Error::Connection);
        }
        self.last_link = Some(authorization.session);
        self.link = Some(Link {
            principal: authorization.principal,
            session: authorization.session,
            last_id: 0,
            digest: [0; 32],
            response: None,
        });
        Ok(())
    }
    /// Revoke only the connection. Keep the transaction, owner and previous installed version.
    pub fn detach(&mut self) {
        self.link = None;
    }

    /// Process exactly one assembled message. Protocol errors revoke this connection.
    /// # Errors
    /// Invalid framing, old identities, skipped IDs or conflicting duplicates require reconnect.
    pub fn process(&mut self, bytes: &[u8]) -> Result<Frame, Error> {
        let result = self.process_inner(bytes);
        if result.is_err() {
            self.detach();
        }
        result
    }
    fn process_inner(&mut self, bytes: &[u8]) -> Result<Frame, Error> {
        let request = Request::decode(bytes)?;
        let link = self.link.as_ref().ok_or(Error::Denied)?;
        if request.link != link.session {
            return Err(Error::Connection);
        }
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if request.id == link.last_id {
            return if digest == link.digest {
                link.response.clone().ok_or(Error::State)
            } else {
                Err(Error::Sequence)
            };
        }
        if request.id != link.last_id.checked_add(1).ok_or(Error::Exhausted)? {
            return Err(Error::Sequence);
        }
        let principal = link.principal;
        let result = self.apply(request.action, principal);
        let response = Response {
            link: request.link,
            id: request.id,
            command: request.action.command(),
            result,
            state: self.state(principal),
        }
        .encode()?;
        let link = self.link.as_mut().ok_or(Error::Connection)?;
        link.last_id = request.id;
        link.digest = digest;
        link.response = Some(response.clone());
        Ok(response)
    }
    fn state(&self, principal: Id) -> State {
        State {
            boot: self.installer.transaction(1).boot,
            head: self.installer.head(),
            progress: self.installer.progress(),
            owned: self.owner == Some(principal),
            max_chunk: MAX_CHUNK_BYTES,
            max_package: MAX_PACKAGE_BYTES,
        }
    }
    fn apply(&mut self, action: Action<'_>, principal: Id) -> Result<(), RemoteError> {
        if action == Action::Status {
            return Ok(());
        }
        let transaction = action.transaction().ok_or(RemoteError::State)?;
        if let Some(p) = self.installer.progress() {
            let terminal = matches!(p.phase, Phase::Committed | Phase::Cancelled);
            let new_begin = matches!(action, Action::Begin { .. }) && p.transaction != transaction;
            if self.owner != Some(principal) && !(terminal && new_begin) {
                return Err(RemoteError::Ownership);
            }
        } else if !matches!(action, Action::Begin { .. }) {
            return Err(RemoteError::Stale);
        }
        let result = match action {
            Action::Status => Ok(()),
            Action::Begin {
                transaction,
                identity,
            } => self.installer.begin(transaction, identity).map(|_| ()),
            Action::Write {
                transaction,
                offset,
                bytes,
            } => self.installer.write(transaction, offset, bytes).map(|_| ()),
            Action::Verify(transaction) => self.installer.verify(transaction).map(|_| ()),
            Action::Commit(transaction) => self.installer.commit(transaction).map(|_| ()),
            Action::Cancel(transaction) => self.installer.cancel(transaction).map(|_| ()),
            Action::Reconcile(transaction) => {
                if self
                    .installer
                    .progress()
                    .is_none_or(|p| p.transaction != transaction)
                {
                    return Err(RemoteError::Stale);
                }
                self.installer.reconcile().map(|_| ())
            }
        };
        // A preparation failure can consume a transaction; ownership must cover that failure too.
        if matches!(action, Action::Begin { .. })
            && self
                .installer
                .progress()
                .is_some_and(|p| p.transaction == transaction)
        {
            self.owner = Some(principal);
        }
        result.map_err(|e| remote_error(&e))
    }
    /// Stable installed read source, for a separate runtime; this service never starts playback.
    /// # Errors
    /// Preserve storage, empty-installation and package validation errors.
    pub fn snapshot(&self) -> Result<Installed<S::Snapshot>, stagemaster_install::Error<S::Error>> {
        self.installer.snapshot()
    }
    /// Current transaction progress without storage I/O. Maintenance owners use it to
    /// prevent leaving an incomplete, failed or uncertain transaction behind.
    #[must_use]
    pub fn progress(&self) -> Option<stagemaster_install::Progress> {
        self.installer.progress()
    }
}
fn remote_error<E>(error: &stagemaster_install::Error<E>) -> RemoteError {
    use stagemaster_install::Error as InstallError;
    match error {
        InstallError::CommitUncertain(_) => RemoteError::Uncertain,
        InstallError::Storage(_) => RemoteError::Storage,
        InstallError::Package(_) => RemoteError::Package,
        InstallError::Code(code) => match code {
            Code::Identity => RemoteError::Identity,
            Code::Stale => RemoteError::Stale,
            Code::Order => RemoteError::Order,
            Code::Busy => RemoteError::Busy,
            Code::Bounds => RemoteError::Bounds,
            Code::Conflict => RemoteError::Conflict,
            Code::Incomplete => RemoteError::Incomplete,
            Code::State => RemoteError::State,
            Code::Uncertain => RemoteError::Uncertain,
            Code::Empty => RemoteError::Empty,
            Code::Exhausted => RemoteError::Exhausted,
            Code::Metadata => RemoteError::Metadata,
        },
    }
}
