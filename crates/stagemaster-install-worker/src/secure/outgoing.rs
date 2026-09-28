use super::{Error, Gateway, Kind, Phase, Sending};
use stagemaster_device_auth::application::Grant;
use stagemaster_device_link::management::ApplicationReceipt;

impl Gateway {
    /// Repeated calls return exactly the same ciphertext until `sent()`, without
    /// advancing a nonce again. Copy it to the bounded record sender once only.
    /// # Errors
    /// Lease, permission, worker and encryption failures revoke the gateway.
    pub fn outbound(&mut self, now: u64) -> Result<Option<&[u8]>, Error> {
        let grant = self.poll(now)?;
        if self.sending.is_none() {
            let result = self.prepare(grant, now);
            self.checked(result)?;
        }
        Ok(self.sending.map(|_| &self.out[..self.length]))
    }
    fn prepare(&mut self, grant: Grant, now: u64) -> Result<(), Error> {
        let sending = if !self.ready && self.endpoint.phase() == Phase::Receiving {
            let context = grant.context();
            let bytes = ApplicationReceipt {
                device: context.device,
                boot: context.boot,
                diagnostic: context.connection,
                session: grant.session(),
                principal: grant.principal(),
                revision: grant.revision(),
                remaining_ms: u32::try_from(grant.expires_at() - now).map_err(|_| Error::Order)?,
            }
            .encode()
            .map_err(Error::Receipt)?;
            self.length = self
                .access
                .seal(Kind::Message, &bytes, &mut self.out, now)?;
            Sending::Ready
        } else if let Some(bytes) = self.endpoint.fragment(now)? {
            self.length = self.access.seal(Kind::Message, bytes, &mut self.out, now)?;
            Sending::Reply
        } else if self.heartbeat {
            self.length = self
                .access
                .seal(Kind::HeartbeatReply, &[], &mut self.out, now)?;
            Sending::Heartbeat
        } else {
            return Ok(());
        };
        self.sending = Some(sending);
        Ok(())
    }
    /// Call after all record fragments were accepted by transport, never after queueing.
    /// This is local delivery progress, NOT remote receipt or an installation commit.
    /// # Errors
    /// Missing outstanding data or any expired/invalid state closes the gateway.
    pub fn sent(&mut self, now: u64) -> Result<(), Error> {
        self.poll(now)?;
        let result = match self.sending.take() {
            Some(Sending::Ready) => {
                self.ready = true;
                Ok(())
            }
            Some(Sending::Heartbeat) => {
                self.heartbeat = false;
                Ok(())
            }
            Some(Sending::Reply) => self
                .endpoint
                .sent(now)
                .map_err(Error::from)
                .and_then(|done| if done { Ok(()) } else { Err(Error::Order) }),
            None => Err(Error::Order),
        };
        self.out.fill(0);
        self.length = 0;
        self.checked(result)
    }
}
