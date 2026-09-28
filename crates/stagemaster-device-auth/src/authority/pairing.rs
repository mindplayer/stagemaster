use super::{
    ADMISSION_MS, Authority, Connection, Error, Evidence, Origin, PAIRING_ATTEMPTS, Phase, Window,
};
use crate::Vault;

impl Authority {
    /// Trusted LOCAL physical action only; no radio command may call this directly.
    /// # Errors
    /// An already open window cannot reset its deadline or consumed attempts.
    pub fn open_pairing(&mut self, now_ms: u64) -> Result<(), Error> {
        self.poll(now_ms)?;
        if self.vault.is_none() {
            return Err(Error::Unavailable);
        }
        if self.window.is_some() {
            return Err(Error::Busy);
        }
        let deadline = self.deadline(now_ms, ADMISSION_MS)?;
        self.window = Some(Window {
            deadline,
            attempts: PAIRING_ATTEMPTS,
        });
        Ok(())
    }
    /// Cancel admission, including any pending attempt; stale success cannot revive it.
    pub fn close_pairing(&mut self) -> Option<Connection> {
        self.window = None;
        if self
            .active
            .as_ref()
            .is_some_and(|a| a.phase == Phase::Pairing)
        {
            return self.active.take().map(|active| active.id);
        }
        None
    }
    /// Call before displaying a passkey. Refusal requires physical disconnection.
    /// # Errors
    /// Each attempt consumes a slot across reconnects; failures never reset the window.
    pub fn begin_pairing(&mut self, connection: Connection, now_ms: u64) -> Result<(), Error> {
        self.check(connection, now_ms)?;
        if self
            .active
            .as_ref()
            .is_none_or(|a| a.phase != Phase::Awaiting)
        {
            return self.close(Error::Denied);
        }
        let Some(window) = self.window.as_mut() else {
            return self.close(Error::PairingClosed);
        };
        if window.attempts == 0 {
            return self.close(Error::Attempts);
        }
        window.attempts -= 1;
        let active = self.active.as_mut().ok_or(Error::Closed)?;
        active.phase = Phase::Pairing;
        active.deadline = active.deadline.min(window.deadline);
        Ok(())
    }
    /// Returns an UNCOMMITTED proposal, never a Grant. Immediately disconnect/revoke
    /// worker epochs, persist it, restore the verified result, then reconnect and prove it.
    /// The fresh principal is device RNG output; known peer identities keep their owner.
    /// # Errors
    /// Late, cancelled, weak or inconsistent pairing results cannot create authority.
    pub fn paired(
        &mut self,
        connection: Connection,
        evidence: &Evidence,
        fresh_principal: [u8; 16],
        now_ms: u64,
    ) -> Result<Vault, Error> {
        self.check(connection, now_ms)?;
        if !evidence.valid(Origin::Pairing)
            || self
                .active
                .as_ref()
                .is_none_or(|a| a.phase != Phase::Pairing)
        {
            return self.close(Error::Denied);
        }
        let vault = self.vault.as_ref().ok_or(Error::Unavailable)?;
        let principal = vault
            .bindings()
            .find(|b| evidence.peer.same_identity(b))
            .map_or(fresh_principal, crate::Binding::principal);
        let proposal = evidence
            .peer
            .binding(principal)
            .and_then(|binding| vault.enroll(binding));
        match proposal {
            Ok(proposal) => {
                self.suspend();
                Ok(proposal)
            }
            Err(error) => self.close(error.into()),
        }
    }
    /// Trusted local management operation. Close live permissions BEFORE the store commits.
    /// # Errors
    /// Refuses missing bindings, unavailable storage and revision exhaustion.
    pub fn revoke(&mut self, principal: [u8; 16]) -> Result<Vault, Error> {
        let proposal = self
            .vault
            .as_ref()
            .ok_or(Error::Unavailable)?
            .revoke(principal)?;
        self.suspend();
        Ok(proposal)
    }
}
