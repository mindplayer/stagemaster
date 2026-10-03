//! Optional fixed-capacity handoff from the original Port to one async transmitter.
mod driver;
mod receiver;
mod state;
mod waiting;

use core::cell::RefCell;
pub use driver::QueuedDriver;
use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};
pub use receiver::Receiver;
use state::{Life, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Busy,
    Closed,
    Line,
    Clock,
    Deadline,
    Exhausted,
    WriteProgress,
    Report,
}

/// One request slot, one completion slot, and an independent priority stop slot.
/// Use a cross-core-safe mutex on hardware; no lock is held across a UART call.
pub struct Queue<M: RawMutex> {
    state: Mutex<M, RefCell<State>>,
}
impl<M: RawMutex> Default for Queue<M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<M: RawMutex> Queue<M> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(RefCell::new(State::new())),
        }
    }

    /// Bind exactly one producer and one consumer. Neither endpoint is cloneable.
    /// # Errors
    /// A previously bound queue can never be silently reused after shutdown/fault.
    pub fn split(&mut self) -> Result<(QueuedDriver<'_, M>, Receiver<'_, M>), Fault> {
        let state = self.state.get_mut().get_mut();
        if state.life != Life::Fresh {
            return Err(Fault::Closed);
        }
        state.life = Life::Open;
        Ok((QueuedDriver { queue: self }, Receiver { queue: self }))
    }

    fn with<T>(&self, f: impl FnOnce(&mut State) -> T) -> T {
        self.state.lock(|cell| f(&mut cell.borrow_mut()))
    }
}
