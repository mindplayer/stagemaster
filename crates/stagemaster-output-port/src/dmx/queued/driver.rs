use super::{
    Fault, Queue,
    state::{Frame, Life},
};
use crate::{Driver, Event, Ticket};
use embassy_sync::blocking_mutex::raw::RawMutex;

/// Synchronous admission only. The original Port remains the output authority.
pub struct QueuedDriver<'a, M: RawMutex> {
    pub(super) queue: &'a Queue<M>,
}
impl<M: RawMutex> Driver for QueuedDriver<'_, M> {
    type Error = Fault;
    fn submit(&mut self, ticket: Ticket, slots: &[u8; 512], until: u64) -> Result<(), Fault> {
        // Both transmission and idle expiry use the same microsecond clock.
        until.checked_mul(1000).ok_or(Fault::Exhausted)?;
        self.queue.with(|s| {
            if s.life != Life::Open {
                return Err(Fault::Closed);
            }
            if let Some(fault) = s.fault {
                return Err(fault);
            }
            if s.frame.is_some() || s.active.is_some() || s.stop.is_some() || s.completion.is_some()
            {
                return Err(Fault::Busy);
            }
            s.frame = Some(Frame {
                ticket,
                slots: *slots,
                until,
            });
            s.wake.wake();
            Ok(())
        })
    }
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), Fault> {
        self.queue.with(|s| {
            if s.life != Life::Open {
                return Err(Fault::Closed);
            }
            s.frame = None;
            s.completion = None;
            s.stop = Some(ticket);
            s.wake.wake();
            Ok(())
        })
    }
    fn poll(&mut self) -> Result<Option<Event>, Fault> {
        self.queue.with(|s| {
            if s.fault_pending {
                s.fault_pending = false;
                return Err(s.fault.expect("pending fault is recorded"));
            }
            if s.life != Life::Open {
                return Err(Fault::Closed);
            }
            Ok(s.completion.take())
        })
    }
}
impl<M: RawMutex> Drop for QueuedDriver<'_, M> {
    fn drop(&mut self) {
        self.queue.with(|s| {
            if s.life == Life::Open {
                s.life = Life::ProducerClosed;
            }
            s.frame = None;
            s.completion = None;
            s.stop = None;
            s.wake.wake();
        });
    }
}
