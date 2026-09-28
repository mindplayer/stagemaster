use super::{C, Channel, Incoming, Peripheral, Problem, REQUEST, RESPONSE, SERVICE, now, wire};
use btleplug::api::{CharPropFlags, Peripheral as _};
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_info::{Description, capability};
use stagemaster_device_link::{management::ApplicationReceipt, secure::Sender};
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Context, HANDSHAKE_BYTES, Handshake, Kind, PLAINTEXT_BYTES,
};
use tokio::time::Instant;
fn entropy(bytes: &mut [u8]) -> Result<(), stagemaster_device_session::Error> {
    getrandom::fill(bytes).map_err(|_| stagemaster_device_session::Error::Entropy)
}
impl Channel {
    pub async fn prepare(
        peripheral: &Peripheral,
        desc: Description,
        config: &Configuration,
    ) -> Result<Self, Problem> {
        validate(&desc, config)?;
        let chars = peripheral.characteristics();
        let find = |uuid, property| {
            chars
                .iter()
                .find(|c| {
                    c.service_uuid == SERVICE && c.uuid == uuid && c.properties.contains(property)
                })
                .cloned()
                .ok_or_else(|| Problem::new(C::Installation))
        };
        let request = find(REQUEST, CharPropFlags::WRITE_WITHOUT_RESPONSE)?;
        let response = find(RESPONSE, CharPropFlags::NOTIFY)?;
        let origin = Instant::now();
        let budget = usize::from(peripheral.mtu().saturating_sub(3).min(244));
        let mut sender = Sender::new(budget, 0).map_err(wire)?;
        let mut incoming = Incoming::spawn(
            peripheral
                .notifications()
                .await
                .map_err(super::super::error)?,
            origin,
            budget,
        );
        peripheral
            .subscribe(&response)
            .await
            .map_err(super::super::error)?;
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
        super::io::send(peripheral, &request, &mut sender, origin, &out[..n]).await?;
        let record = incoming.receive().await?;
        handshake.read(record.bytes(), now(origin)).map_err(wire)?;
        let mut secure = handshake.finish(now(origin)).map_err(wire)?;
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = secure
            .confirmation(&mut cipher, now(origin))
            .map_err(wire)?;
        let confirmation_started = now(origin);
        super::io::send(peripheral, &request, &mut sender, origin, &cipher[..n]).await?;
        let record = incoming.receive().await?;
        secure.confirm(record.bytes(), now(origin)).map_err(wire)?;
        let proof = secure
            .peer(now(origin))
            .map_err(wire)?
            .ok_or_else(|| Problem::new(C::Installation))?;
        let record = incoming.receive().await?;
        let mut plain = [0; PLAINTEXT_BYTES];
        let message = secure
            .open(record.bytes(), &mut plain, now(origin))
            .map_err(wire)?;
        if message.kind != Kind::Message {
            return Err(Problem::new(C::Installation));
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
        Ok(Self {
            receipt,
            secure,
            sender,
            incoming,
            request,
            origin,
            received_at: now(origin),
            until: confirmation_started
                .checked_add(u64::from(receipt.remaining_ms))
                .ok_or_else(|| Problem::new(C::Installation))?,
            pending: None,
            usable: true,
        })
    }
}

fn validate(desc: &Description, config: &Configuration) -> Result<(), Problem> {
    if config.role() != Role::Controller
        || config.device() != desc.device
        || desc.authentication != stagemaster_device_session::AUTHENTICATION
        || !desc.declares(capability::INSTALLATION)
        || desc.limits.transfer_version != 1
        || usize::from(desc.limits.message_bytes) != stagemaster_device_session::MAX_PAYLOAD
    {
        return Err(Problem::new(C::Installation));
    }
    Ok(())
}
