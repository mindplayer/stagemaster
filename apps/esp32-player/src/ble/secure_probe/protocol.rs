//! Probe application: only return received data or heartbeat acknowledgement.
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, Error, HANDSHAKE_BYTES, HANDSHAKE_MS, Handshake, Kind,
    PLAINTEXT_BYTES, SecretKey,
};

#[allow(clippy::large_enum_variant)] // Fixed single connection; avoid another heap allocation.
enum Phase {
    Idle,
    Handshake(Handshake),
    Confirm(Channel),
    Active(Channel),
    Closed,
}
pub(super) struct Protocol {
    phase: Phase,
    until: u64,
}
impl Protocol {
    pub const fn new() -> Self {
        Self {
            phase: Phase::Idle,
            until: 0,
        }
    }
    pub fn start(&mut self, context: Context, key: &SecretKey, now: u64) -> Result<(), Error> {
        if !matches!(self.phase, Phase::Idle) {
            return Err(Error::State);
        }
        self.until = now.checked_add(HANDSHAKE_MS).ok_or(Error::Clock)?;
        self.phase = Phase::Handshake(Handshake::respond(context, key, super::entropy, now)?);
        Ok(())
    }
    pub fn poll(&mut self, now: u64) -> Result<(), Error> {
        match &mut self.phase {
            Phase::Idle => Ok(()),
            Phase::Handshake(_) if now < self.until => Ok(()),
            Phase::Handshake(_) => Err(Error::Expired),
            Phase::Confirm(channel) | Phase::Active(channel) => channel.poll(now),
            Phase::Closed => Err(Error::Closed),
        }
    }
    pub fn receive(
        &mut self,
        bytes: &[u8],
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<usize, Error> {
        self.poll(now)?;
        let phase = core::mem::replace(&mut self.phase, Phase::Closed);
        match phase {
            Phase::Handshake(mut handshake) => {
                handshake.read(bytes, now)?;
                let mut reply = [0; HANDSHAKE_BYTES];
                let n = handshake.write(&mut reply, super::now())?;
                out[..n].copy_from_slice(&reply[..n]);
                self.phase = Phase::Confirm(handshake.finish(super::now())?);
                Ok(n)
            }
            Phase::Confirm(mut channel) => {
                channel.confirm(bytes, now)?;
                let n = channel.confirmation(out, super::now())?;
                assert!(channel.peer(super::now())?.is_some());
                self.phase = Phase::Active(channel);
                esp_println::println!(
                    "安全无线实验相互确认；没有安装权限；heap={} peak={}",
                    esp_alloc::HEAP.used(),
                    esp_alloc::HEAP.stats().max_usage
                );
                Ok(n)
            }
            Phase::Active(mut channel) => {
                let mut plaintext = [0; PLAINTEXT_BYTES];
                let record = channel.open(bytes, &mut plaintext, now)?;
                let kind = match record.kind {
                    Kind::Message => Kind::Message,
                    Kind::Heartbeat => Kind::HeartbeatReply,
                    Kind::HeartbeatReply => return Err(Error::State),
                };
                let result = channel.seal(kind, record.payload, out, super::now());
                plaintext.fill(0);
                let n = result?;
                self.phase = Phase::Active(channel);
                Ok(n)
            }
            _ => Err(Error::State),
        }
    }
}
