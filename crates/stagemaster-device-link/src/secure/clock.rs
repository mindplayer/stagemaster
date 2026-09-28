use super::{Error, RECORD_MS};

pub(super) struct Clock {
    last: u64,
    until: Option<u64>,
    closed: bool,
}
impl Clock {
    pub const fn new(now: u64) -> Self {
        Self {
            last: now,
            until: None,
            closed: false,
        }
    }
    pub fn poll(&mut self, now: u64) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if now < self.last {
            return Err(Error::Clock);
        }
        self.last = now;
        if self.until.is_some_and(|until| now >= until) {
            return Err(Error::Expired);
        }
        Ok(())
    }
    pub fn arm(&mut self, now: u64) -> Result<(), Error> {
        self.poll(now)?;
        if self.until.is_some() {
            return Err(Error::State);
        }
        self.until = Some(now.checked_add(RECORD_MS).ok_or(Error::Clock)?);
        Ok(())
    }
    pub const fn idle(&mut self) {
        self.until = None;
    }
    pub const fn close(&mut self) {
        self.closed = true;
        self.until = None;
    }
}
