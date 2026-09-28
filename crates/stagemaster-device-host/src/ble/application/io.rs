use super::{C, Channel, Characteristic, Peripheral, Problem, now, wire};
use btleplug::api::{Peripheral as _, WriteType};
use stagemaster_device_link::secure::Sender;
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};
use tokio::time::Instant;
use zeroize::Zeroizing;

pub(super) async fn send(
    peripheral: &Peripheral,
    request: &Characteristic,
    sender: &mut Sender,
    origin: Instant,
    bytes: &[u8],
) -> Result<(), Problem> {
    sender.queue(bytes, now(origin)).map_err(wire)?;
    while let Some(packet) = sender.fragment(now(origin)).map_err(wire)? {
        // Sequential native futures respect platform credit. No parallel writes.
        peripheral
            .write(request, packet.bytes(), WriteType::WithoutResponse)
            .await
            .map_err(super::super::error)?;
        sender.sent(now(origin)).map_err(wire)?;
    }
    Ok(())
}
impl Channel {
    pub async fn write(&mut self, peripheral: &Peripheral, bytes: &[u8]) -> Result<(), Problem> {
        let result = self.send(peripheral, Kind::Message, bytes).await;
        self.checked(result)
    }
    async fn send(
        &mut self,
        peripheral: &Peripheral,
        kind: Kind,
        bytes: &[u8],
    ) -> Result<(), Problem> {
        self.check()?;
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self
            .secure
            .seal(kind, bytes, &mut cipher, now(self.origin))
            .map_err(wire)?;
        send(
            peripheral,
            &self.request,
            &mut self.sender,
            self.origin,
            &cipher[..n],
        )
        .await?;
        self.check()
    }
    fn decode(&mut self, bytes: &[u8]) -> Result<(Kind, Vec<u8>), Problem> {
        self.check()?;
        let mut plain = Zeroizing::new([0; PLAINTEXT_BYTES]);
        let record = self
            .secure
            .open(bytes, &mut plain, now(self.origin))
            .map_err(wire)?;
        self.received_at = now(self.origin);
        Ok((record.kind, record.payload.to_vec()))
    }
    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, Problem> {
        let result = (|| {
            self.check()?;
            if self.pending.is_some() {
                return Ok(self.pending.take());
            }
            let Some(record) = self.incoming.next()? else {
                return Ok(None);
            };
            let (kind, bytes) = self.decode(record.bytes())?;
            if kind != Kind::Message {
                return Err(Problem::new(C::Protocol));
            }
            Ok(Some(bytes))
        })();
        self.checked(result)
    }
    pub async fn heartbeat(&mut self, peripheral: &Peripheral) -> Result<(), Problem> {
        let result = self.heartbeat_inner(peripheral).await;
        self.checked(result)
    }
    async fn heartbeat_inner(&mut self, peripheral: &Peripheral) -> Result<(), Problem> {
        self.send(peripheral, Kind::Heartbeat, &[]).await?;
        loop {
            let record = self.incoming.receive().await?;
            let (kind, bytes) = self.decode(record.bytes())?;
            match kind {
                Kind::HeartbeatReply => return Ok(()),
                Kind::Message if self.pending.is_none() => self.pending = Some(bytes),
                _ => return Err(Problem::new(C::Protocol)),
            }
        }
    }
}
