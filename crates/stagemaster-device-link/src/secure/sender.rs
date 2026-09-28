use super::{Error, Record, clock::Clock, limit};
use crate::management::{Packet, Serial};

struct Pending {
    record: Record,
    offset: usize,
    flight: Option<usize>,
}
pub struct Sender {
    serial: Serial,
    pending: Option<Pending>,
    limit: usize,
    clock: Clock,
}
impl Sender {
    /// # Errors
    /// Only GATT value budgets 20..=244 are supported.
    pub fn new(packet_bytes: usize, now: u64) -> Result<Self, Error> {
        Ok(Self {
            serial: Serial::default(),
            pending: None,
            limit: limit(packet_bytes)?,
            clock: Clock::new(now),
        })
    }
    pub fn close(&mut self) {
        self.clock.close();
        if let Some(pending) = &mut self.pending {
            pending.record.bytes.fill(0);
        }
        self.pending = None;
    }
    /// # Errors
    /// Time rollback/expiry closes the direction; idle polling never renews a deadline.
    pub fn poll(&mut self, now: u64) -> Result<(), Error> {
        let result = self.clock.poll(now);
        self.checked(result)
    }
    /// Reserve one whole opaque message. No allocation, retry or unbounded queue.
    /// # Errors
    /// Invalid size, occupied sender or time failure permanently closes it.
    pub fn queue(&mut self, bytes: &[u8], now: u64) -> Result<(), Error> {
        let result = (|| {
            self.poll(now)?;
            if self.pending.is_some() {
                return Err(Error::State);
            }
            let record = Record::new(bytes)?;
            self.clock.arm(now)?;
            self.pending = Some(Pending {
                record,
                offset: 0,
                flight: None,
            });
            Ok(())
        })();
        self.checked(result)
    }
    /// Repeated calls return the same fragment until `sent` acknowledges acceptance.
    /// # Errors
    /// Time or exhausted serial state closes this direction.
    pub fn fragment(&mut self, now: u64) -> Result<Option<Packet>, Error> {
        let result = (|| {
            self.poll(now)?;
            let Some(pending) = &mut self.pending else {
                return Ok(None);
            };
            let length = *pending
                .flight
                .get_or_insert_with(|| self.limit.min(pending.record.length - pending.offset));
            self.serial
                .encode(&pending.record.bytes[pending.offset..pending.offset + length])
                .map(Some)
                .map_err(Error::Fragment)
        })();
        self.checked(result)
    }
    /// Call only after the exact pending fragment was successfully submitted to I/O.
    /// True means all bytes submitted, not authenticated receipt or installation success.
    /// # Errors
    /// Missing outstanding fragment or time/serial error permanently closes it.
    pub fn sent(&mut self, now: u64) -> Result<bool, Error> {
        let result = (|| {
            self.poll(now)?;
            let pending = self.pending.as_mut().ok_or(Error::State)?;
            let length = pending.flight.take().ok_or(Error::State)?;
            self.serial.advance().map_err(Error::Fragment)?;
            pending.offset += length;
            let complete = pending.offset == pending.record.length;
            if complete {
                self.pending = None;
                self.clock.idle();
            }
            Ok(complete)
        })();
        self.checked(result)
    }
    fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
}
