use crate::{Channel, Error, MAX_RECORD_BYTES, RecordIo, now, receive, wire};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};
use zeroize::Zeroizing;

impl<R: RecordIo> Channel<R> {
    /// # Errors
    /// Any failure or cancellation invalidates this session, with uncertain delivery.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.begin()?;
        let result = self.send(Kind::Message, bytes).await;
        self.finish(result)
    }
    async fn send(&mut self, kind: Kind, bytes: &[u8]) -> Result<(), Error> {
        let mut cipher = [0; CIPHERTEXT_BYTES];
        let n = self
            .secure
            .seal(kind, bytes, &mut cipher, now(self.origin))
            .map_err(wire)?;
        self.io.send(&cipher[..n]).await
    }
    fn decode(&mut self, bytes: &[u8]) -> Result<(Kind, Vec<u8>), Error> {
        if !(1..=MAX_RECORD_BYTES).contains(&bytes.len()) {
            return Err(Error::Bounds);
        }
        let mut plain = Zeroizing::new([0; PLAINTEXT_BYTES]);
        let record = self
            .secure
            .open(bytes, &mut plain, now(self.origin))
            .map_err(wire)?;
        self.received_at = now(self.origin);
        Ok((record.kind, record.payload.to_vec()))
    }
    /// Nonblocking complete application message retrieval.
    /// # Errors
    /// Refuses failed carriers, stale sessions, tampering, replay and unexpected message kinds.
    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let result = (|| {
            self.check()?;
            if self.pending.is_some() {
                return Ok(self.pending.take());
            }
            let Some(record) = self.io.try_receive()? else {
                return Ok(None);
            };
            let (kind, bytes) = self.decode(&record)?;
            if kind != Kind::Message {
                return Err(Error::Denied);
            }
            Ok(Some(bytes))
        })();
        self.checked(result)
    }
    /// # Errors
    /// Requires the encrypted heartbeat reply; at most one intervening message is retained.
    /// Cancellation invalidates the session, including any outstanding reply.
    pub async fn heartbeat(&mut self) -> Result<(), Error> {
        self.begin()?;
        let result = self.heartbeat_inner().await;
        self.finish(result)
    }
    async fn heartbeat_inner(&mut self) -> Result<(), Error> {
        self.send(Kind::Heartbeat, &[]).await?;
        loop {
            let record = receive(&mut self.io).await?;
            let (kind, bytes) = self.decode(&record)?;
            match kind {
                Kind::HeartbeatReply => return Ok(()),
                Kind::Message if self.pending.is_none() => self.pending = Some(bytes),
                _ => return Err(Error::Denied),
            }
        }
    }
}
