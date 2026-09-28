mod confirmation;
mod records;

use crate::{Context, Error, context::Clock};
use snow::TransportState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Message,
    Heartbeat,
    HeartbeatReply,
}
impl Kind {
    fn byte(self) -> u8 {
        match self {
            Self::Message => 1,
            Self::Heartbeat => 2,
            Self::HeartbeatReply => 3,
        }
    }
    fn parse(byte: u8) -> Result<Self, Error> {
        match byte {
            1 => Ok(Self::Message),
            2 => Ok(Self::Heartbeat),
            3 => Ok(Self::HeartbeatReply),
            _ => Err(Error::Invalid),
        }
    }
    fn valid(self, payload: &[u8]) -> bool {
        match self {
            Self::Message => !payload.is_empty(),
            Self::Heartbeat | Self::HeartbeatReply => payload.is_empty(),
        }
    }
}
pub struct Record<'a> {
    pub kind: Kind,
    pub payload: &'a [u8],
}

/// Proof of live key possession only. Authorizers must separately establish trust,
/// scopes and current validity; this is not a cloud claim or an install grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerProof {
    context: Context,
    public_key: [u8; 32],
    transcript: [u8; 32],
}
impl PeerProof {
    #[must_use]
    pub const fn context(&self) -> Context {
        self.context
    }
    #[must_use]
    pub const fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }
    #[must_use]
    pub const fn transcript(&self) -> &[u8; 32] {
        &self.transcript
    }
    #[must_use]
    pub fn session(&self) -> [u8; 16] {
        let mut session = [0; 16];
        session.copy_from_slice(&self.transcript[..16]);
        session
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    SendConfirm,
    AwaitConfirm,
    SendReady,
    AwaitReady,
    Established,
    Closed,
}

pub struct Channel {
    noise: Option<TransportState>,
    proof: PeerProof,
    phase: Phase,
    clock: Clock,
    confirmed_at: Option<u64>,
}
impl Channel {
    pub(crate) fn new(
        noise: TransportState,
        context: Context,
        public_key: [u8; 32],
        transcript: [u8; 32],
        initiator: bool,
        clock: Clock,
    ) -> Self {
        Self {
            noise: Some(noise),
            proof: PeerProof {
                context,
                public_key,
                transcript,
            },
            phase: if initiator {
                Phase::SendConfirm
            } else {
                Phase::AwaitConfirm
            },
            clock,
            confirmed_at: None,
        }
    }
    pub fn close(&mut self) {
        self.noise = None;
        self.phase = Phase::Closed;
    }
    /// Check fixed deadlines even while no packets arrive. Polling never renews them.
    /// # Errors
    /// Time rollback, overflow, expiry or an already closed channel cannot be recovered.
    pub fn poll(&mut self, now: u64) -> Result<(), Error> {
        if self.phase == Phase::Closed {
            return Err(Error::Closed);
        }
        let result = self.clock.check(now);
        self.finish_result(result)
    }
    /// Return a time-checked proof only after both sides have confirmed the session.
    /// # Errors
    /// Report a closed, expired or invalid clock; never return a stale proof.
    pub fn peer(&mut self, now: u64) -> Result<Option<PeerProof>, Error> {
        self.poll(now)?;
        Ok((self.phase == Phase::Established).then_some(self.proof))
    }
    fn established(&mut self, now: u64) -> Result<(), Error> {
        self.poll(now)?;
        if self.phase != Phase::Established {
            return Err(Error::State);
        }
        Ok(())
    }
    fn finish_result<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
}
