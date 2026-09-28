use crate::{
    Channel, Context, Entropy, Error, SUITE, SecretKey,
    context::Clock,
    crypto::{Resolver, error},
};
use alloc::boxed::Box;
use snow::{Builder, HandshakeState};
use zeroize::Zeroize;

pub const HANDSHAKE_BYTES: usize = 96;
/// Fixed IK handshake. It carries no installation commands or cloud permissions.
pub struct Handshake {
    noise: Option<HandshakeState>,
    context: Context,
    clock: Clock,
}
impl Handshake {
    /// Start using a device public key obtained from a trusted source, not a broadcast.
    /// # Errors
    /// Reject invalid context, key material, time or unsupported fixed primitives.
    pub fn initiate(
        context: Context,
        local: &SecretKey,
        device: [u8; 32],
        entropy: Entropy,
        now: u64,
    ) -> Result<Self, Error> {
        if device == [0; 32] {
            return Err(Error::Invalid);
        }
        Self::build(context, local, Some(device), entropy, now)
    }
    /// Receive an initiator's identity proof; permission must be checked separately.
    /// # Errors
    /// Reject invalid context, time or unsupported fixed primitives.
    pub fn respond(
        context: Context,
        local: &SecretKey,
        entropy: Entropy,
        now: u64,
    ) -> Result<Self, Error> {
        Self::build(context, local, None, entropy, now)
    }
    fn build(
        context: Context,
        local: &SecretKey,
        remote: Option<[u8; 32]>,
        entropy: Entropy,
        now: u64,
    ) -> Result<Self, Error> {
        let clock = Clock::new(now)?;
        let prologue = context.encode()?;
        let builder =
            Builder::with_resolver(SUITE.parse().map_err(error)?, Box::new(Resolver(entropy)))
                .local_private_key(local.bytes())
                .map_err(error)?
                .prologue(&prologue)
                .map_err(error)?;
        let noise = match &remote {
            Some(key) => builder
                .remote_public_key(key)
                .map_err(error)?
                .build_initiator(),
            None => builder.build_responder(),
        }
        .map_err(error)?;
        Ok(Self {
            noise: Some(noise),
            context,
            clock,
        })
    }
    /// Produce one whole handshake message. The adapter handles its fragmentation.
    /// # Errors
    /// Any order, time or crypto error permanently closes this handshake.
    pub fn write(&mut self, out: &mut [u8; HANDSHAKE_BYTES], now: u64) -> Result<usize, Error> {
        let result = (|| {
            self.clock.check(now)?;
            let noise = self.noise.as_mut().ok_or(Error::Closed)?;
            if !noise.is_my_turn() || noise.is_handshake_finished() {
                return Err(Error::State);
            }
            noise.write_message(&[], out).map_err(error)
        })();
        if result.is_err() {
            self.noise = None;
            out.zeroize();
        }
        result
    }
    /// Accept exactly the expected fixed message, without application payload.
    /// # Errors
    /// Wrong keys/context, malformed, repeated or late messages close the handshake.
    pub fn read(&mut self, bytes: &[u8], now: u64) -> Result<(), Error> {
        let result = (|| {
            self.clock.check(now)?;
            let noise = self.noise.as_mut().ok_or(Error::Closed)?;
            if noise.is_my_turn() || noise.is_handshake_finished() {
                return Err(Error::State);
            }
            let expected = if noise.is_initiator() { 48 } else { 96 };
            if bytes.len() != expected {
                return Err(Error::Invalid);
            }
            noise.read_message(bytes, &mut []).map_err(error)?;
            Ok(())
        })();
        if result.is_err() {
            self.noise = None;
        }
        result
    }
    /// Move into the confirmation phase; this does not yet expose a peer proof.
    /// # Errors
    /// Reject an incomplete, closed or expired handshake.
    pub fn finish(mut self, now: u64) -> Result<Channel, Error> {
        self.clock.check(now)?;
        let noise = self.noise.take().ok_or(Error::Closed)?;
        if !noise.is_handshake_finished() {
            return Err(Error::State);
        }
        let remote = noise
            .get_remote_static()
            .ok_or(Error::State)?
            .try_into()
            .map_err(|_| Error::State)?;
        let hash = noise
            .get_handshake_hash()
            .try_into()
            .map_err(|_| Error::State)?;
        let initiator = noise.is_initiator();
        let transport = noise.into_transport_mode().map_err(error)?;
        Ok(Channel::new(
            transport,
            self.context,
            remote,
            hash,
            initiator,
            self.clock,
        ))
    }
}
