use super::{BUFFER_BYTES, Error, MAX_RECORD_BYTES, RECORD_HEADER};

/// Untrusted handshake/ciphertext bytes, never a permission or verified plaintext.
pub struct Record {
    pub(super) bytes: [u8; BUFFER_BYTES],
    pub(super) length: usize,
}
impl Record {
    pub(super) const fn empty() -> Self {
        Self {
            bytes: [0; BUFFER_BYTES],
            length: 0,
        }
    }
    pub(super) fn new(payload: &[u8]) -> Result<Self, Error> {
        if !(1..=MAX_RECORD_BYTES).contains(&payload.len()) {
            return Err(Error::Bounds);
        }
        let len = u16::try_from(payload.len()).map_err(|_| Error::Bounds)?;
        let mut record = Self::empty();
        record.bytes[0] = 1;
        record.bytes[2..4].copy_from_slice(&len.to_be_bytes());
        record.length = RECORD_HEADER + payload.len();
        record.bytes[RECORD_HEADER..record.length].copy_from_slice(payload);
        Ok(record)
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[RECORD_HEADER..self.length]
    }
}

pub(super) fn expected(bytes: &[u8]) -> Result<usize, Error> {
    if bytes[0] != 1 || bytes[1] != 0 {
        return Err(Error::Format);
    }
    let length = usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
    if !(1..=MAX_RECORD_BYTES).contains(&length) {
        return Err(Error::Bounds);
    }
    Ok(length + RECORD_HEADER)
}
