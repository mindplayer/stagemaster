use crate::{Error, RecordIo, now, receive, wire};
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_info::Description;
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, HANDSHAKE_BYTES, Handshake, PeerProof,
};
use tokio::time::Instant;
fn entropy(bytes: &mut [u8]) -> Result<(), stagemaster_device_session::Error> {
    getrandom::fill(bytes).map_err(|_| stagemaster_device_session::Error::Entropy)
}
pub(crate) async fn authenticate(
    io: &mut impl RecordIo,
    desc: Description,
    config: &Configuration,
    origin: Instant,
) -> Result<(Channel, PeerProof, u64), Error> {
    desc.validate().map_err(wire)?;
    if config.role() != Role::Controller
        || config.device() != desc.device
        || desc.authentication != stagemaster_device_session::AUTHENTICATION
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
    Ok((secure, proof, confirmation_started))
}
