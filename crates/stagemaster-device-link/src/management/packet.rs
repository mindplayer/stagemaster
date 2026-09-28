use super::{Error, HEADER_BYTES, PACKET_BYTES};

pub struct Packet {
    bytes: [u8; PACKET_BYTES],
    len: usize,
}
impl Packet {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

/// One direction of one physical connection. Call advance only after send acceptance.
pub struct Serial {
    next: u32,
}
impl Default for Serial {
    fn default() -> Self {
        Self { next: 1 }
    }
}
impl Serial {
    /// # Errors
    /// Empty/oversized packets and exhausted sequence space are rejected.
    pub fn encode(&self, payload: &[u8]) -> Result<Packet, Error> {
        if payload.is_empty() || payload.len() > PACKET_BYTES - HEADER_BYTES {
            return Err(Error::Limits);
        }
        self.next.checked_add(1).ok_or(Error::Exhausted)?;
        let mut packet = Packet {
            bytes: [0; PACKET_BYTES],
            len: payload.len() + HEADER_BYTES,
        };
        packet.bytes[..HEADER_BYTES].copy_from_slice(&self.next.to_le_bytes());
        packet.bytes[HEADER_BYTES..packet.len].copy_from_slice(payload);
        Ok(packet)
    }
    /// # Errors
    /// Never wraps to an earlier fragment number.
    pub fn advance(&mut self) -> Result<(), Error> {
        self.next = self.next.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(())
    }
    /// # Errors
    /// Drops/duplicates/reordering are errors. Caller must close the connection on error.
    pub fn receive<'a>(&mut self, bytes: &'a [u8]) -> Result<&'a [u8], Error> {
        if !(HEADER_BYTES + 1..=PACKET_BYTES).contains(&bytes.len()) {
            return Err(Error::Limits);
        }
        let serial = u32::from_le_bytes(core::array::from_fn(|i| bytes[i]));
        if serial != self.next {
            return Err(Error::Sequence);
        }
        self.advance()?;
        Ok(&bytes[HEADER_BYTES..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrap_is_never_emitted_or_accepted() {
        let mut sequence = Serial { next: u32::MAX };
        assert!(matches!(sequence.encode(&[1]), Err(Error::Exhausted)));
        assert_eq!(
            sequence.receive(&[255, 255, 255, 255, 1]),
            Err(Error::Exhausted)
        );
        assert_eq!(sequence.advance(), Err(Error::Exhausted));
    }
}
