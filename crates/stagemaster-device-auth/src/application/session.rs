use super::{DevelopmentPermit, Error, Grant};
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, Kind, PLAINTEXT_BYTES, Record,
};
use zeroize::Zeroize;

/// Owns the channel so a caller cannot renew or replace proof behind admission.
pub struct Session {
    channel: Channel,
    grant: Option<Grant>,
}
impl Session {
    /// Admission consumes a confirmed channel. Failure drops all its authority.
    /// # Errors
    /// Refuses unconfirmed, expired, mismatched or untrusted key possession.
    pub fn admit(
        mut channel: Channel,
        permit: DevelopmentPermit,
        context: Context,
        now: u64,
    ) -> Result<Self, Error> {
        let proof = channel.peer(now)?.ok_or(Error::Denied)?;
        let grant = permit.admit(proof, context, now)?;
        Ok(Self {
            channel,
            grant: Some(grant),
        })
    }
    pub fn revoke(&mut self) {
        self.grant = None;
        self.channel.close();
    }
    /// Poll even while idle or while the storage worker is busy.
    /// # Errors
    /// Revocation, fixed permission expiry, clock and security failures are terminal.
    pub fn grant(&mut self, now: u64) -> Result<Grant, Error> {
        let result = (|| {
            let grant = self.grant.ok_or(Error::Closed)?;
            let proof = self.channel.peer(now)?.ok_or(Error::Denied)?;
            if now >= grant.until {
                return Err(Error::Expired);
            }
            if proof.context() != grant.context || proof.session() != grant.session {
                return Err(Error::Denied);
            }
            Ok(grant)
        })();
        self.checked(result)
    }
    /// # Errors
    /// A record cannot refresh an expired permission or resurrect a closed session.
    /// Every failure clears the caller's plaintext buffer.
    pub fn open<'a>(
        &mut self,
        bytes: &[u8],
        out: &'a mut [u8; PLAINTEXT_BYTES],
        now: u64,
    ) -> Result<Record<'a>, Error> {
        if let Err(error) = self.grant(now) {
            out.zeroize();
            return Err(error);
        }
        let result = self.channel.open(bytes, out, now).map_err(Error::from);
        self.checked(result)
    }
    /// # Errors
    /// Fixed permission and secure lease are checked before advancing the nonce.
    pub fn seal(
        &mut self,
        kind: Kind,
        payload: &[u8],
        out: &mut [u8; CIPHERTEXT_BYTES],
        now: u64,
    ) -> Result<usize, Error> {
        if let Err(error) = self.grant(now) {
            out.zeroize();
            return Err(error);
        }
        let result = self
            .channel
            .seal(kind, payload, out, now)
            .map_err(Error::from);
        self.checked(result)
    }
    fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.revoke();
        }
        result
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.revoke();
    }
}
