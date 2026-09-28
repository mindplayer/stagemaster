//! Host-side correlation. The transport owns deadlines and drops this on disconnect.
use crate::{Code, HEARTBEAT_MS, HELLO, PACKET_BYTES, PING, PROTOCOL_VERSION, Packet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Packet(Code),
    Correlation,
    Rejected(u8),
    State,
    Diagnostics,
}

/// Exactly one request at a time. Never retransmit a heartbeat on this protocol.
#[derive(Default)]
pub struct Client {
    session: u64,
    sequence: u32,
    pending: Option<Packet>,
}
impl Client {
    /// Correlation token of the accepted handshake, never authentication evidence.
    #[must_use]
    pub const fn session_id(&self) -> Option<u64> {
        if self.session == 0 {
            None
        } else {
            Some(self.session)
        }
    }

    /// # Errors
    /// An unanswered request or exhausted sequence requires a new connection.
    pub fn request(&mut self) -> Result<[u8; PACKET_BYTES], Error> {
        if self.pending.is_some() {
            return Err(Error::State);
        }
        let sequence = if self.session == 0 {
            0
        } else {
            self.sequence.checked_add(1).ok_or(Error::State)?
        };
        let packet = Packet {
            kind: if self.session == 0 { HELLO } else { PING },
            code: 0,
            session: self.session,
            sequence,
            value: 0,
        };
        self.pending = Some(packet);
        Ok(packet.encode())
    }

    /// Only a matching application receipt proves liveness. Invalid replies keep
    /// the request pending; callers may read the cached reply again, never rewrite.
    /// # Errors
    /// Rejects malformed, stale, wrong-session and failed receipts.
    pub fn accept(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let request = self.pending.ok_or(Error::State)?;
        let reply = Packet::decode(bytes).map_err(Error::Packet)?;
        if reply.kind != request.kind | 0x80
            || reply.sequence != request.sequence
            || reply.session == 0
            || (self.session != 0 && reply.session != self.session)
        {
            return Err(Error::Correlation);
        }
        if reply.code != 0 {
            return Err(Error::Rejected(reply.code));
        }
        if reply.value != HEARTBEAT_MS {
            return Err(Error::Correlation);
        }
        self.session = reply.session;
        self.sequence = reply.sequence;
        self.pending = None;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostics {
    pub self_test: bool,
    pub output_disabled: bool,
    pub uptime_ms: u32,
    pub ticks: u32,
    pub heap_used: u32,
    pub heap_free: u32,
}
impl Diagnostics {
    /// # Errors
    /// Unknown protocol/flags and contradictory heap values are not diagnostic proof.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != PACKET_BYTES
            || bytes[0] != PROTOCOL_VERSION
            || bytes[1] & !3 != 0
            || bytes[2] != 0
            || bytes[3] != 0
        {
            return Err(Error::Diagnostics);
        }
        let get = |offset| u32::from_le_bytes(core::array::from_fn(|i| bytes[offset + i]));
        let value = Self {
            self_test: bytes[1] & 1 != 0,
            output_disabled: bytes[1] & 2 != 0,
            uptime_ms: get(4),
            ticks: get(8),
            heap_used: get(12),
            heap_free: get(16),
        };
        if value.heap_used.checked_add(value.heap_free) != Some(128 * 1024) {
            return Err(Error::Diagnostics);
        }
        Ok(value)
    }
}
