//! Bounded transport-independent diagnostic sessions. No playback or I/O ownership.
#![no_std]
#![forbid(unsafe_code)]

pub const PACKET_BYTES: usize = 20;
pub const PROTOCOL_VERSION: u8 = 1;
pub const HEARTBEAT_MS: u32 = 2_000;
pub const EXPIRY_MS: u64 = 6_000;
pub const HELLO: u8 = 1;
pub const PING: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Code {
    Ok = 0,
    Malformed = 1,
    Version = 2,
    Session = 3,
    Order = 4,
    Expired = 5,
    Unsupported = 6,
    Clock = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet {
    pub kind: u8,
    pub code: u8,
    pub session: u64,
    pub sequence: u32,
    pub value: u32,
}

impl Packet {
    #[must_use]
    pub fn encode(self) -> [u8; PACKET_BYTES] {
        let mut bytes = [0; PACKET_BYTES];
        bytes[0] = PROTOCOL_VERSION;
        bytes[1] = self.kind;
        bytes[2] = self.code;
        bytes[4..12].copy_from_slice(&self.session.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.value.to_le_bytes());
        bytes
    }

    /// Decode either a request or response. Request-only constraints belong to Session.
    /// # Errors
    /// Rejects unsupported versions, length, reserved bits, and unknown result codes.
    pub fn decode(bytes: &[u8]) -> Result<Self, Code> {
        if bytes.len() != PACKET_BYTES {
            return Err(Code::Malformed);
        }
        if bytes[0] != PROTOCOL_VERSION {
            return Err(Code::Version);
        }
        if bytes[3] != 0 || bytes[2] > Code::Clock as u8 {
            return Err(Code::Malformed);
        }
        Ok(Self {
            kind: bytes[1],
            code: bytes[2],
            session: u64::from_le_bytes(core::array::from_fn(|i| bytes[4 + i])),
            sequence: u32::from_le_bytes(core::array::from_fn(|i| bytes[12 + i])),
            value: u32::from_le_bytes(core::array::from_fn(|i| bytes[16 + i])),
        })
    }
}

/// One transport connection. Drop on disconnect; give each new connection a fresh nonzero nonce.
pub struct Session {
    nonce: u64,
    last_ms: u64,
    last_seen_ms: u64,
    sequence: u32,
    established: bool,
    expired: bool,
}

impl Session {
    /// # Errors
    /// Zero is reserved for an unestablished session.
    pub fn new(nonce: u64, now_ms: u64) -> Result<Self, Code> {
        if nonce == 0 {
            return Err(Code::Session);
        }
        Ok(Self {
            nonce,
            last_ms: now_ms,
            last_seen_ms: now_ms,
            sequence: 0,
            established: false,
            expired: false,
        })
    }

    /// # Errors
    /// Rejects backwards host time, without mutating the session.
    pub fn poll(&mut self, now_ms: u64) -> Result<bool, Code> {
        if now_ms < self.last_ms {
            return Err(Code::Clock);
        }
        self.last_ms = now_ms;
        self.expired |= now_ms - self.last_seen_ms >= EXPIRY_MS;
        Ok(self.expired)
    }

    /// Returns a correlated diagnostic reply. Invalid requests never renew liveness.
    #[must_use]
    pub fn receive(&mut self, bytes: &[u8], now_ms: u64) -> Packet {
        let parsed = Packet::decode(bytes);
        let request = parsed.unwrap_or(Packet {
            kind: 0,
            code: 0,
            session: 0,
            sequence: 0,
            value: 0,
        });
        let result = self.accept(parsed, now_ms);
        Packet {
            kind: request.kind | 0x80,
            code: result.map_or_else(|error| error as u8, |()| Code::Ok as u8),
            session: self.nonce,
            sequence: request.sequence,
            value: HEARTBEAT_MS,
        }
    }

    fn accept(&mut self, request: Result<Packet, Code>, now_ms: u64) -> Result<(), Code> {
        if self.poll(now_ms)? {
            return Err(Code::Expired);
        }
        let packet = request?;
        if packet.code != 0 || packet.value != 0 {
            return Err(Code::Malformed);
        }
        match packet.kind {
            HELLO if !self.established && packet.session == 0 && packet.sequence == 0 => {
                self.established = true;
            }
            HELLO => return Err(Code::Session),
            PING => {
                if !self.established || packet.session != self.nonce {
                    return Err(Code::Session);
                }
                if packet.sequence <= self.sequence {
                    return Err(Code::Order);
                }
                self.sequence = packet.sequence;
            }
            _ => return Err(Code::Unsupported),
        }
        self.last_seen_ms = now_ms;
        Ok(())
    }
}
