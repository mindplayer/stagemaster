use crate::{Command, Error, Frame, GROUP, MAX_FRAME_BYTES};

pub(crate) const HEADER: usize = 8;
pub(crate) fn header(bytes: &[u8]) -> Result<(usize, Command, bool), Error> {
    if bytes.len() < HEADER {
        return Err(Error::Malformed);
    }
    if bytes[0] & !3 != 8 || bytes[1] != 0 {
        return Err(Error::Version);
    }
    if u16::from_be_bytes([bytes[4], bytes[5]]) != GROUP {
        return Err(Error::Version);
    }
    let command = Command::read(bytes[7])?;
    let op = bytes[0] & 3;
    if (command == Command::Status && op > 1) || (command != Command::Status && op < 2) {
        return Err(Error::Malformed);
    }
    let length = HEADER + usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
    if !(HEADER + 1..=MAX_FRAME_BYTES).contains(&length) {
        return Err(Error::Bounds);
    }
    Ok((length, command, op & 1 == 1))
}

/// One ordered connection, one message at a time. On timeout/partial send, discard the connection.
pub struct Assembler {
    frame: Frame,
    expected: Option<usize>,
    poisoned: bool,
}
impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}
impl Assembler {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frame: Frame::empty(),
            expected: None,
            poisoned: false,
        }
    }
    /// # Errors
    /// Empty/overflowed/invalid input poisons this assembler; use a fresh transport connection.
    pub fn push(&mut self, bytes: &[u8]) -> Result<bool, Error> {
        if self.poisoned {
            return Err(Error::Connection);
        }
        let result = self.append(bytes);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn append(&mut self, bytes: &[u8]) -> Result<bool, Error> {
        if bytes.is_empty() || self.expected == Some(self.frame.length) {
            return Err(Error::State);
        }
        let end = self
            .frame
            .length
            .checked_add(bytes.len())
            .ok_or(Error::Bounds)?;
        if end > MAX_FRAME_BYTES {
            return Err(Error::Bounds);
        }
        self.frame.bytes[self.frame.length..end].copy_from_slice(bytes);
        self.frame.length = end;
        if self.expected.is_none() && end >= HEADER {
            self.expected = Some(header(self.frame.bytes())?.0);
        }
        if self.expected.is_some_and(|n| end > n) {
            return Err(Error::Bounds);
        }
        Ok(self.expected == Some(end))
    }
    /// Return only a whole frame; an incomplete frame must not be reset and followed by retries.
    pub fn take(&mut self) -> Option<Frame> {
        if self.poisoned || self.expected != Some(self.frame.length) {
            return None;
        }
        self.expected = None;
        Some(core::mem::replace(&mut self.frame, Frame::empty()))
    }
}
