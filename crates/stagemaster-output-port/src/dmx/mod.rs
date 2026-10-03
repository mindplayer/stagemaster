//! Single DMX transaction. The caller owns scheduling, queueing and port authority.
mod deadline;
mod transfer;
use core::future::Future;

pub const BAUD: u32 = 250_000;
pub const BREAK_BITS: u32 = 30;
pub const MARK_US: u64 = 16;
pub const FRAME_BYTES: usize = 513;
/// Nominal wire time for our 120 us break, 16 us MAB and 513 8N2 characters.
pub const MIN_FRAME_US: u64 = 120 + MARK_US + FRAME_BYTES as u64 * 44;

/// A sole owner of the configured 250 kbit/s, 8N2 UART and RS485 direction pin.
/// Async methods must yield promptly. `disable` must synchronously gate the line.
/// No automatic retransmission, hidden queue, allocation or pin sharing is allowed.
pub trait Line {
    type Error;
    fn disable(&mut self);
    fn enable(&mut self);
    /// Must restore UART marking polarity when this future is cancelled.
    fn break_signal(&mut self, bits: u32) -> impl Future<Output = Result<(), Self::Error>>;
    /// May write only a prefix; completion means FIFO admission, not wire completion.
    fn write(&mut self, bytes: &[u8]) -> impl Future<Output = Result<usize, Self::Error>>;
    /// Wait for FIFO AND final shift register/stop bits. Never clear pending bytes
    /// and call that successful transmission. Cancellation may leave bytes queued.
    fn drain(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Must use the same monotonic epoch as the original Port's millisecond clock.
/// The timer must wake independently of the producer and not complete early.
pub trait Clock {
    fn now_us(&self) -> u64;
    fn wait_until_us(&self, deadline: u64) -> impl Future<Output = ()>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Budget,
    Clock,
    Deadline,
    Exhausted,
    WriteProgress,
    Line(E),
}

/// Serial, bounded transactions. This is the async worker side of Driver;
/// never block a synchronous `Driver::submit` waiting for it.
pub struct Transmitter<L: Line, C: Clock> {
    line: L,
    clock: C,
    timeout_us: u64,
    last_us: u64,
}

impl<L: Line, C: Clock> Transmitter<L, C> {
    /// Immediately gates off the line, including on invalid configuration.
    /// # Errors
    /// Rejects a total operation budget outside the supported bounded range.
    pub fn new(mut line: L, clock: C, timeout_us: u64) -> Result<Self, Error<L::Error>> {
        line.disable();
        if !(MIN_FRAME_US..=1_000_000).contains(&timeout_us) {
            return Err(Error::Budget);
        }
        let last_us = clock.now_us();
        Ok(Self {
            line,
            clock,
            timeout_us,
            last_us,
        })
    }
}

impl<L: Line, C: Clock> Drop for Transmitter<L, C> {
    fn drop(&mut self) {
        self.line.disable();
    }
}

struct Gate<'a, L: Line> {
    line: &'a mut L,
    armed: bool,
}
impl<'a, L: Line> Gate<'a, L> {
    fn new(line: &'a mut L) -> Self {
        Self { line, armed: true }
    }
}
impl<L: Line> Drop for Gate<'_, L> {
    fn drop(&mut self) {
        if self.armed {
            self.line.disable();
        }
    }
}
