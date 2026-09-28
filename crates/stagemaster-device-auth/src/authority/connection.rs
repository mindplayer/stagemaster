use super::{
    ADMISSION_MS, Active, Authority, Connection, Error, Evidence, Grant, LEASE_MS, Origin, Phase,
    Security,
};
use core::num::NonZeroU32;

impl Authority {
    /// Device RNG supplies a fresh session per physical connection, never a peer input.
    /// # Errors
    /// Refuses concurrent links, unavailable credentials, invalid entropy or counter wrap.
    pub fn connect(&mut self, session: [u8; 16], now_ms: u64) -> Result<Connection, Error> {
        self.poll(now_ms)?;
        if self.vault.is_none() {
            return Err(Error::Unavailable);
        }
        if self.active.is_some() {
            return Err(Error::Busy);
        }
        if session == [0; 16] || self.previous_session == Some(session) {
            return Err(Error::Invalid);
        }
        let deadline = self.deadline(now_ms, ADMISSION_MS)?;
        let epoch = self.next_epoch.checked_add(1).ok_or(Error::Exhausted)?;
        let connection = Connection(NonZeroU32::new(epoch).ok_or(Error::Exhausted)?);
        self.next_epoch = epoch;
        self.previous_session = Some(session);
        self.active = Some(Active {
            id: connection,
            session,
            deadline,
            phase: Phase::Awaiting,
        });
        Ok(connection)
    }
    /// Accept only the stack's successful encryption-resumption event on this connection.
    /// # Errors
    /// Weak, fresh-pairing, unknown or mismatched credentials close this link.
    pub fn resumed(
        &mut self,
        connection: Connection,
        evidence: &Evidence,
        now_ms: u64,
    ) -> Result<Grant, Error> {
        self.check(connection, now_ms)?;
        if !evidence.valid(Origin::Resumed)
            || self
                .active
                .as_ref()
                .is_none_or(|a| a.phase != Phase::Awaiting)
        {
            return self.close(Error::Denied);
        }
        let principal = self.vault.as_ref().and_then(|v| {
            v.bindings()
                .find(|binding| evidence.peer.matches(binding))
                .map(crate::Binding::principal)
        });
        let Some(principal) = principal else {
            return self.close(Error::Denied);
        };
        let deadline = self.deadline(now_ms, LEASE_MS)?;
        let active = self.active.as_mut().ok_or(Error::Closed)?;
        active.phase = Phase::Granted(principal);
        active.deadline = deadline;
        self.grant(connection, Security::Authenticated, now_ms)
    }
    /// Sample the actual stack security before every privileged dispatch.
    /// # Errors
    /// A downgrade or deadline closes authority; merely reading it never renews its lease.
    pub fn grant(
        &mut self,
        connection: Connection,
        security: Security,
        now_ms: u64,
    ) -> Result<Grant, Error> {
        self.check(connection, now_ms)?;
        if security != Security::Authenticated {
            return self.close(Error::Denied);
        }
        let active = self.active.as_ref().ok_or(Error::Closed)?;
        let Phase::Granted(principal) = active.phase else {
            return Err(Error::Denied);
        };
        Ok(Grant {
            connection,
            principal,
            session: active.session,
        })
    }
    /// Call only after the existing diagnostic Session accepts a valid HELLO/PING.
    /// # Errors
    /// Expired or downgraded links cannot be renewed or revived.
    pub fn heartbeat(
        &mut self,
        connection: Connection,
        security: Security,
        now_ms: u64,
    ) -> Result<(), Error> {
        self.grant(connection, security, now_ms)?;
        let deadline = self.deadline(now_ms, LEASE_MS)?;
        self.active.as_mut().ok_or(Error::Closed)?.deadline = deadline;
        Ok(())
    }
}
