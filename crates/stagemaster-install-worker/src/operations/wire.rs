use super::{Connection, Error};
use crate::ManagedWorker;
use stagemaster_device_auth::application::{Grant, Scope};
use stagemaster_install::Storage;
use stagemaster_runtime::PlaybackPolicy;
use stagemaster_runtime_protocol::{Access, Frame, Offer, Peer, Ready, Request, Response};

const _: () = assert!(
    stagemaster_runtime_protocol::MAX_MESSAGE_BYTES == stagemaster_device_session::MAX_PAYLOAD
);

impl Connection {
    /// Consume an offer already decrypted in this exact admitted session. Once only.
    /// The adapter must close on failed/cancelled delivery and keep heartbeats independent.
    /// # Errors
    /// Invalid order/version, stale access or a bad offer closes this input connection.
    pub fn negotiate<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        bytes: &[u8],
        now: u64,
        live: impl FnMut(u64) -> Option<Grant>,
    ) -> Result<Frame, Error> {
        self.poll(device, now, live)?;
        let result = (|| {
            if self.wire_ready || self.last.is_some() {
                return Err(Error::Negotiation);
            }
            let offer = Offer::decode(bytes).map_err(Error::Protocol)?;
            let version = offer.select().map_err(Error::Protocol)?;
            let g = self.grant;
            let frame = Ready {
                peer: Peer {
                    device: g.context().device,
                    boot: g.context().boot,
                    connection: g.context().connection,
                    session: g.session(),
                    principal: g.principal(),
                    permission_revision: g.revision(),
                },
                version,
                message_bytes: offer.message_bytes,
                access: Access {
                    observe: g.permissions().contains(Scope::Observe),
                    control: g.permissions().contains(Scope::Control),
                    installation: g.permissions().contains(Scope::Installation),
                },
                remaining_ms: u32::try_from(g.expires_at().saturating_sub(now))
                    .map_err(|_| Error::Obsolete)?,
            }
            .encode()
            .map_err(Error::Protocol)?;
            self.wire_ready = true;
            Ok(frame)
        })();
        if result.is_err() {
            self.close(device).map_err(Error::Runtime)?;
        }
        result
    }

    /// Process one complete decrypted runtime message on the storage/runtime owner.
    /// Never invoke slow Load from a radio callback; callers must separately own a bounded queue.
    /// # Errors
    /// Negotiation/decode/encode errors close input; a misrouted foreign session is refused
    /// without evicting the current owner. Existing operation authority is rechecked.
    pub fn process_message<S: Storage, P: PlaybackPolicy>(
        &mut self,
        device: &mut ManagedWorker<S, P>,
        bytes: &[u8],
        clock: impl FnMut() -> u64,
        live: impl FnMut(u64) -> Option<Grant>,
    ) -> Result<Frame, Error> {
        let result = (|| {
            if !self.wire_ready {
                return Err(Error::Negotiation);
            }
            let request = Request::decode(bytes).map_err(Error::Protocol)?;
            let reply = self.process(device, request, clock, live)?;
            Response::from(reply).encode().map_err(Error::Protocol)
        })();
        if result.is_err() && !matches!(result, Err(Error::Identity)) {
            self.close(device).map_err(Error::Runtime)?;
        }
        result
    }
}
