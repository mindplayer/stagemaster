use super::{EXCHANGE_MS, Error, Gateway, after, gateway::Sending};
use stagemaster_device_auth::application::{Grant, Scope};
use stagemaster_device_session::Kind;
use stagemaster_runtime_protocol::{Access, Frame, Offer, Peer, Ready};

impl Gateway {
    /// One immutable ciphertext until all transport fragments have been accepted.
    /// Keep calling while work is pending, so heartbeats do not wait for storage.
    /// # Errors
    /// Deadlines and encryption failures close input authority.
    pub fn outbound(&mut self, now: u64) -> Result<Option<&[u8]>, Error> {
        self.poll(now)?;
        if self.sending.is_none() {
            let result = self.prepare(now);
            self.checked(result)?;
        }
        Ok(self.sending.map(|_| &self.out[..self.length]))
    }
    fn prepare(&mut self, now: u64) -> Result<(), Error> {
        let sending = if self.heartbeat_until.is_some() {
            self.length = self
                .access
                .seal(Kind::HeartbeatReply, &[], &mut self.out, now)?;
            Sending::Heartbeat
        } else if let Some(frame) = self.pending.as_ref().and_then(|p| p.frame.as_ref()) {
            self.length = self
                .access
                .seal(Kind::Message, frame.bytes(), &mut self.out, now)?;
            Sending::Reply
        } else {
            return Ok(());
        };
        self.sending = Some((sending, after(now, EXCHANGE_MS)?));
        Ok(())
    }
    /// A local transport completion, not a remote acknowledgement or physical output proof.
    /// # Errors
    /// Missing in-flight data, backwards time and expired exchanges close the session.
    pub fn sent(&mut self, now: u64) -> Result<(), Error> {
        self.poll(now)?;
        let result = match self.sending.take() {
            Some((Sending::Heartbeat, _)) => {
                self.heartbeat_until = None;
                Ok(())
            }
            Some((Sending::Reply, _)) => self.delivered(),
            None => Err(Error::Order),
        };
        self.out.fill(0);
        self.length = 0;
        self.checked(result)
    }
}

pub(super) fn check_ready(
    frame: &Frame,
    offer: Offer,
    grant: Grant,
    opened_ms: u64,
    now: u64,
) -> Result<Frame, Error> {
    let mut ready = Ready::decode(frame.bytes())?;
    // remaining_ms was captured by the worker; delivery may be delayed. Identity and
    // scopes must match exactly, and no receipt may exceed the original permit budget.
    let expected = Ready {
        peer: Peer {
            device: grant.context().device,
            boot: grant.context().boot,
            connection: grant.context().connection,
            session: grant.session(),
            principal: grant.principal(),
            permission_revision: grant.revision(),
        },
        version: offer.select()?,
        message_bytes: offer.message_bytes,
        access: Access {
            observe: grant.permissions().contains(Scope::Observe),
            control: grant.permissions().contains(Scope::Control),
            installation: grant.permissions().contains(Scope::Installation),
        },
        remaining_ms: u32::try_from(grant.expires_at().saturating_sub(opened_ms))
            .map_err(|_| Error::Order)?,
    };
    ready.correlate(offer, &expected)?;
    ready.remaining_ms =
        u32::try_from(grant.expires_at().saturating_sub(now)).map_err(|_| Error::Order)?;
    ready.encode().map_err(Error::from)
}
