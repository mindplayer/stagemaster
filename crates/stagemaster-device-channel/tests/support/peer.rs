use super::{config, context, entropy, now};
use stagemaster_device_auth::application::{DevelopmentPermit, Permissions, Role, Session};
use stagemaster_device_channel::{Error, RecordIo, receive};
use stagemaster_device_info::Description;
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, HANDSHAKE_BYTES, Handshake, Kind, PLAINTEXT_BYTES,
};
use tokio::time::Instant;

pub struct Peer<R: RecordIo> {
    pub io: R,
    pub secure: Session,
    pub origin: Instant,
}
impl<R: RecordIo> Peer<R> {
    pub async fn accept(io: R, desc: Description, stale_receipt: bool) -> Result<Self, Error> {
        let mut peer = Self::authenticate(io, desc).await?;
        peer.ready(desc, stale_receipt).await?;
        Ok(peer)
    }
    pub async fn authenticate(io: R, desc: Description) -> Result<Self, Error> {
        Self::authenticate_with(io, desc, None, Instant::now()).await
    }
    pub async fn authenticate_with(
        mut io: R,
        desc: Description,
        permissions: Option<Permissions>,
        origin: Instant,
    ) -> Result<Self, Error> {
        let config = config(Role::Device);
        let mut handshake =
            Handshake::respond(context(desc), config.key(), entropy, now(origin)).map_err(wire)?;
        handshake
            .read(&receive(&mut io).await?, now(origin))
            .map_err(wire)?;
        let mut out = [0; HANDSHAKE_BYTES];
        let n = handshake.write(&mut out, now(origin)).map_err(wire)?;
        io.send(&out[..n]).await?;
        let mut channel = handshake.finish(now(origin)).map_err(wire)?;
        channel
            .confirm(&receive(&mut io).await?, now(origin))
            .map_err(wire)?;
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = channel
            .confirmation(&mut cipher, now(origin))
            .map_err(wire)?;
        io.send(&cipher[..n]).await?;
        let permit = permissions
            .map_or_else(
                || config.permit(),
                |scopes| {
                    DevelopmentPermit::scoped(
                        config.device(),
                        config.trusted_key(),
                        config.principal(),
                        config.revision(),
                        config.duration_ms(),
                        scopes,
                    )
                },
            )
            .map_err(wire)?;
        let secure = Session::admit(channel, permit, context(desc), now(origin)).map_err(wire)?;
        Ok(Self { io, secure, origin })
    }
    pub async fn ready(&mut self, desc: Description, stale_receipt: bool) -> Result<(), Error> {
        let grant = self.secure.grant(now(self.origin)).map_err(wire)?;
        let receipt = ApplicationReceipt {
            device: desc.device,
            boot: desc.boot,
            diagnostic: desc.session,
            session: if stale_receipt {
                [99; 16]
            } else {
                grant.session()
            },
            principal: grant.principal(),
            revision: grant.revision(),
            remaining_ms: 60_000,
        }
        .encode()
        .unwrap();
        self.send(Kind::Message, &receipt).await
    }
    pub async fn receive(&mut self) -> Result<(Kind, Vec<u8>), Error> {
        let record = receive(&mut self.io).await?;
        let mut plain = [0; PLAINTEXT_BYTES];
        let opened = self
            .secure
            .open(&record, &mut plain, now(self.origin))
            .map_err(wire)?;
        Ok((opened.kind, opened.payload.to_vec()))
    }
    pub async fn send(&mut self, kind: Kind, bytes: &[u8]) -> Result<(), Error> {
        let cipher = self.seal(kind, bytes)?;
        self.io.send(&cipher).await
    }
    pub fn seal(&mut self, kind: Kind, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self
            .secure
            .seal(kind, bytes, &mut cipher, now(self.origin))
            .map_err(wire)?;
        Ok(cipher[..n].to_vec())
    }
}
fn wire(error: impl std::fmt::Display) -> Error {
    Error::Protocol(error.to_string())
}
