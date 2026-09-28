use super::{BUFFER_BYTES, Error, RECORD_HEADER, Record, clock::Clock, limit, record::expected};
use crate::management::{HEADER_BYTES, Serial};

pub struct Receiver {
    serial: Serial,
    record: Record,
    expected: Option<usize>,
    limit: usize,
    clock: Clock,
}
impl Receiver {
    /// One receive direction per connection; packet budget includes the serial prefix.
    /// # Errors
    /// Only GATT value budgets 20..=244 are supported.
    pub fn new(packet_bytes: usize, now: u64) -> Result<Self, Error> {
        Ok(Self {
            serial: Serial::default(),
            record: Record::empty(),
            expected: None,
            limit: limit(packet_bytes)?,
            clock: Clock::new(now),
        })
    }
    pub fn close(&mut self) {
        self.clock.close();
        self.record.bytes.fill(0);
        self.record.length = 0;
        self.expected = None;
    }
    /// # Errors
    /// Poll even when no fragments arrive; expired/invalid time permanently closes it.
    pub fn poll(&mut self, now: u64) -> Result<(), Error> {
        let result = self.clock.poll(now);
        if result.is_err() {
            self.close();
        }
        result
    }
    /// Reassemble at most one record, preserving the serial across complete records.
    /// # Errors
    /// Any malformed/oversized/out-of-order input or time failure closes this direction.
    pub fn push(&mut self, packet: &[u8], now: u64) -> Result<Option<Record>, Error> {
        let result = self.append(packet, now);
        if result.is_err() {
            self.close();
        }
        result
    }
    fn append(&mut self, packet: &[u8], now: u64) -> Result<Option<Record>, Error> {
        self.poll(now)?;
        if packet.len() > self.limit + HEADER_BYTES {
            return Err(Error::Bounds);
        }
        let payload = self.serial.receive(packet).map_err(Error::Fragment)?;
        if self.record.length == 0 {
            self.clock.arm(now)?;
        }
        let end = self
            .record
            .length
            .checked_add(payload.len())
            .ok_or(Error::Bounds)?;
        if end > BUFFER_BYTES {
            return Err(Error::Bounds);
        }
        self.record.bytes[self.record.length..end].copy_from_slice(payload);
        self.record.length = end;
        if self.expected.is_none() && end >= RECORD_HEADER {
            self.expected = Some(expected(&self.record.bytes)?);
        }
        if self.expected.is_some_and(|length| end > length) {
            return Err(Error::Bounds);
        }
        if self.expected == Some(end) {
            self.expected = None;
            self.clock.idle();
            Ok(Some(core::mem::replace(&mut self.record, Record::empty())))
        } else {
            Ok(None)
        }
    }
}
