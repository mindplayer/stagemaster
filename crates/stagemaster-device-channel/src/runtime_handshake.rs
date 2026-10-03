use crate::{
    Channel, Error, RecordIo, admission::Admission, authentication::authenticate, now, receive,
    wire,
};
use stagemaster_device_auth::application::Configuration;
use stagemaster_device_info::{Description, capability};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};
use stagemaster_runtime_protocol::{Access, Offer, Peer, Ready};
use std::time::Duration;
use tokio::time::Instant;
use zeroize::Zeroizing;

impl<R: RecordIo> Channel<R> {
    /// Explicit runtime negotiation. Expected rights are trusted local expectations,
    /// never a request that upgrades the remote device's installation-only permit.
    /// # Errors
    /// Refuse absent runtime capability, mismatched readiness and the fixed handshake deadline.
    /// Cancellation drops the owned carrier; no downgrade or automatic retry is performed.
    pub async fn prepare_runtime(
        mut io: R,
        desc: Description,
        config: &Configuration,
        expected: Access,
    ) -> Result<Self, Error> {
        let origin = Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            establish(&mut io, desc, config, expected, origin),
        )
        .await
        .unwrap_or(Err(Error::Timeout));
        match result {
            Ok((secure, receipt, until)) => Ok(Self {
                admission: Admission::Runtime(receipt),
                secure,
                io,
                origin,
                received_at: now(origin),
                until,
                pending: None,
                usable: true,
                application_started: false,
            }),
            Err(error) => {
                io.close();
                Err(error)
            }
        }
    }
}
async fn establish(
    io: &mut impl RecordIo,
    desc: Description,
    config: &Configuration,
    expected: Access,
    origin: Instant,
) -> Result<(stagemaster_device_session::Channel, Ready, u64), Error> {
    if !desc.declares(capability::RUNTIME_APPLICATION) || (!expected.observe && !expected.control) {
        return Err(Error::Denied);
    }
    let (mut secure, proof, confirmation_started) = authenticate(io, desc, config, origin).await?;
    let offer = Offer::current();
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = secure
        .seal(
            Kind::Message,
            offer.encode().map_err(wire)?.bytes(),
            &mut cipher,
            now(origin),
        )
        .map_err(wire)?;
    io.send(&cipher[..n]).await?;
    let record = receive(io).await?;
    let mut plain = Zeroizing::new([0; PLAINTEXT_BYTES]);
    let message = secure
        .open(&record, &mut plain, now(origin))
        .map_err(wire)?;
    if message.kind != Kind::Message {
        return Err(Error::Denied);
    }
    let receipt = Ready::decode(message.payload).map_err(wire)?;
    receipt
        .correlate(
            offer,
            &Ready {
                peer: Peer {
                    device: desc.device,
                    boot: desc.boot,
                    connection: desc.session,
                    session: proof.session(),
                    principal: config.principal(),
                    permission_revision: config.revision(),
                },
                version: 1,
                message_bytes: offer.message_bytes,
                access: expected,
                remaining_ms: config.duration_ms(),
            },
        )
        .map_err(wire)?;
    let until = confirmation_started
        .checked_add(u64::from(receipt.remaining_ms))
        .ok_or(Error::Denied)?;
    if !io.healthy() || now(origin) >= until {
        return Err(Error::Denied);
    }
    Ok((secure, receipt, until))
}
