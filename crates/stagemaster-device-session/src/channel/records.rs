use super::{Channel, Kind, Record};
use crate::{CIPHERTEXT_BYTES, Error, MAX_PAYLOAD, PLAINTEXT_BYTES, crypto::error};
use zeroize::Zeroize;

impl Channel {
    /// Encrypt a bounded record. A send does not renew this peer's liveness lease.
    /// # Errors
    /// Invalid type/length, unconfirmed session, time or crypto errors close the channel.
    pub fn seal(
        &mut self,
        kind: Kind,
        bytes: &[u8],
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<usize, Error> {
        let mut plaintext = [0; PLAINTEXT_BYTES];
        let result = (|| {
            self.established(now)?;
            if bytes.len() > MAX_PAYLOAD || !kind.valid(bytes) {
                return Err(Error::Invalid);
            }
            plaintext[0] = kind.byte();
            plaintext[1..=bytes.len()].copy_from_slice(bytes);
            self.noise
                .as_mut()
                .ok_or(Error::Closed)?
                .write_message(&plaintext[..=bytes.len()], out)
                .map_err(error)
        })();
        plaintext.zeroize();
        if result.is_err() {
            out.zeroize();
        }
        self.finish_result(result)
    }
    /// Authenticate before exposing plaintext. Only a valid incoming record renews the lease.
    /// # Errors
    /// Truncation, overflow, replay, corruption, unexpected kinds or expired sessions close it.
    pub fn open<'a>(
        &mut self,
        bytes: &[u8],
        out: &'a mut [u8; PLAINTEXT_BYTES],
        now: u64,
    ) -> Result<Record<'a>, Error> {
        let result = (|| {
            self.established(now)?;
            if !(17..=CIPHERTEXT_BYTES).contains(&bytes.len()) {
                return Err(Error::Invalid);
            }
            let length = self
                .noise
                .as_mut()
                .ok_or(Error::Closed)?
                .read_message(bytes, out)
                .map_err(error)?;
            let kind = Kind::parse(out[0])?;
            if !kind.valid(&out[1..length]) {
                return Err(Error::Invalid);
            }
            self.clock.received(now)?;
            Ok((kind, length))
        })();
        if result.is_err() {
            out.zeroize();
        }
        let (kind, length) = self.finish_result(result)?;
        Ok(Record {
            kind,
            payload: &out[1..length],
        })
    }
}
