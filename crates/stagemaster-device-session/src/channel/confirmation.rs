use super::{Channel, Phase};
use crate::{CIPHERTEXT_BYTES, Error, crypto::error};
use zeroize::Zeroize;

impl Channel {
    /// Emit the initiator confirmation or responder acknowledgement, without business data.
    /// # Errors
    /// Wrong order, expired clock or crypto failure closes the channel.
    pub fn confirmation(
        &mut self,
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<usize, Error> {
        let result = (|| {
            self.poll(now)?;
            let (kind, next) = match self.phase {
                Phase::SendConfirm => (0xf0, Phase::AwaitReady),
                Phase::SendReady => (0xf1, Phase::Established),
                _ => return Err(Error::State),
            };
            if next == Phase::Established {
                self.clock
                    .establish(self.confirmed_at.ok_or(Error::State)?, now)?;
            }
            let mut payload = [0; 33];
            payload[0] = kind;
            payload[1..].copy_from_slice(&self.proof.transcript);
            let result = self
                .noise
                .as_mut()
                .ok_or(Error::Closed)?
                .write_message(&payload, out)
                .map_err(error);
            payload.zeroize();
            let length = result?;
            self.phase = next;
            Ok(length)
        })();
        if result.is_err() {
            out.zeroize();
        }
        self.finish_result(result)
    }
    /// Check the encrypted transcript confirmation before exposing the peer identity.
    /// # Errors
    /// Wrong transcript, role, length, key or deadline permanently closes the channel.
    pub fn confirm(&mut self, bytes: &[u8], now: u64) -> Result<(), Error> {
        let mut payload = [0; 33];
        let result = (|| {
            self.poll(now)?;
            let (kind, next) = match self.phase {
                Phase::AwaitConfirm => (0xf0, Phase::SendReady),
                Phase::AwaitReady => (0xf1, Phase::Established),
                _ => return Err(Error::State),
            };
            if bytes.len() != 49 {
                return Err(Error::Invalid);
            }
            let length = self
                .noise
                .as_mut()
                .ok_or(Error::Closed)?
                .read_message(bytes, &mut payload)
                .map_err(error)?;
            if length != 33 || payload[0] != kind || payload[1..] != self.proof.transcript {
                return Err(Error::Crypto);
            }
            if next == Phase::Established {
                self.clock.received(now)?;
            } else {
                // Do not extend the fixed handshake/confirmation deadline.
                self.confirmed_at = Some(now);
            }
            self.phase = next;
            Ok(())
        })();
        payload.zeroize();
        self.finish_result(result)
    }
}
