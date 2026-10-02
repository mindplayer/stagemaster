use crate::{Channel, Error, RecordIo, now, receive, wire};
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_info::{Description, capability};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Context, HANDSHAKE_BYTES, Handshake, Kind, PLAINTEXT_BYTES,
};
use std::time::Duration;
use tokio::time::Instant;
use zeroize::Zeroizing;

fn entropy(bytes: &mut [u8]) -> Result<(), stagemaster_device_session::Error> {
    getrandom::fill(bytes).map_err(|_| stagemaster_device_session::Error::Entropy)
}
impl<R: RecordIo> Channel<R> {
    /// Consumes an established record carrier and trusted local configuration.
    /// # Errors
    /// Refuses bad keys, context, capabilities, permission receipts or fixed handshake timeout.
    /// Cancellation drops the owned carrier; no partially authenticated state is reusable.
    pub async fn prepare(
        mut io: R,
        desc: Description,
        config: &Configuration,
    ) -> Result<Self, Error> {
        let origin = Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            establish(&mut io, desc, config, origin),
        )
        .await
        .unwrap_or(Err(Error::Timeout));
        match result {
            Ok((secure, receipt, until)) => Ok(Self {
                receipt,
                secure,
                io,
                origin,
                received_at: now(origin),
                until,
                pending: None,
                usable: true,
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
    origin: Instant,
) -> Result<(stagemaster_device_session::Channel, ApplicationReceipt, u64), Error> {
    if config.role() != Role::Controller
        || config.device() != desc.device
        || desc.authentication != stagemaster_device_session::AUTHENTICATION
        || !desc.declares(capability::INSTALLATION)
        || desc.limits.transfer_version != 1
        || usize::from(desc.limits.message_bytes) != stagemaster_device_session::MAX_PAYLOAD
    {
        return Err(Error::Denied);
    }
    let context = Context {
        device: desc.device,
        boot: desc.boot,
        connection: desc.session,
    };
    let mut handshake = Handshake::initiate(
        context,
        config.key(),
        config.trusted_key(),
        entropy,
        now(origin),
    )
    .map_err(wire)?;
    let mut out = [0; HANDSHAKE_BYTES];
    let n = handshake.write(&mut out, now(origin)).map_err(wire)?;
    io.send(&out[..n]).await?;
    handshake
        .read(&receive(io).await?, now(origin))
        .map_err(wire)?;
    let mut secure = handshake.finish(now(origin)).map_err(wire)?;
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = secure
        .confirmation(&mut cipher, now(origin))
        .map_err(wire)?;
    let confirmation_started = now(origin);
    io.send(&cipher[..n]).await?;
    secure
        .confirm(&receive(io).await?, now(origin))
        .map_err(wire)?;
    let proof = secure
        .peer(now(origin))
        .map_err(wire)?
        .ok_or(Error::Denied)?;
    let record = receive(io).await?;
    let mut plain = Zeroizing::new([0; PLAINTEXT_BYTES]);
    let message = secure
        .open(&record, &mut plain, now(origin))
        .map_err(wire)?;
    if message.kind != Kind::Message {
        return Err(Error::Denied);
    }
    let receipt = ApplicationReceipt::decode(message.payload).map_err(wire)?;
    receipt
        .correlate(&ApplicationReceipt {
            device: desc.device,
            boot: desc.boot,
            diagnostic: desc.session,
            session: proof.session(),
            principal: config.principal(),
            revision: config.revision(),
            remaining_ms: config.duration_ms(),
        })
        .map_err(wire)?;
    let until = confirmation_started
        .checked_add(u64::from(receipt.remaining_ms))
        .ok_or(Error::Denied)?;
    if !io.healthy() || now(origin) >= until {
        return Err(Error::Denied);
    }
    Ok((secure, receipt, until))
}
