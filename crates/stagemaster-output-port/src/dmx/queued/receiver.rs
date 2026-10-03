use super::{
    Fault, Queue,
    state::{Life, Work},
};
use crate::dmx::{Clock, Error, Line, Transmitter};
use embassy_sync::blocking_mutex::raw::RawMutex;

/// The sole consumer. Run on an independently scheduled task; never block Driver.
pub struct Receiver<'a, M: RawMutex> {
    pub(super) queue: &'a Queue<M>,
}
impl<M: RawMutex> Receiver<'_, M> {
    /// Own both endpoints' cleanup for the lifetime of this future. Cancellation,
    /// including before the first poll, drops the transmitter and closes the queue.
    pub async fn run<L: Line, C: Clock>(self, mut tx: Transmitter<L, C>) {
        let mut idle_until = None;
        while let Some(work) = self.next(&mut tx, idle_until).await {
            let kind = work.kind();
            let result = match work {
                Work::Frame(frame) => {
                    let result = self
                        .interruptible(kind, tx.send(frame.ticket, &frame.slots, frame.until))
                        .await;
                    idle_until = if matches!(result, Some(Ok(crate::Event::Sent(_)))) {
                        if let Some(until) = frame.until.checked_mul(1000) {
                            Some(until)
                        } else {
                            tx.line.disable();
                            self.queue.with(|s| s.fail(Fault::Exhausted));
                            None
                        }
                    } else {
                        None
                    };
                    result
                }
                Work::Stop(ticket) => {
                    idle_until = None;
                    self.interruptible(kind, tx.quiesce(ticket)).await
                }
            };
            self.queue
                .with(|s| s.finish(kind, result.map(|r| r.map_err(|e| classify(&e)))));
        }
    }
}
impl<M: RawMutex> Drop for Receiver<'_, M> {
    fn drop(&mut self) {
        self.queue.with(|s| {
            if s.life == Life::Open {
                s.life = Life::ConsumerClosed;
            }
            s.stop = None;
            s.active = None;
            s.fail(Fault::Closed);
        });
    }
}
pub(super) fn classify<E>(error: &Error<E>) -> Fault {
    match error {
        Error::Budget | Error::Exhausted => Fault::Exhausted,
        Error::Clock => Fault::Clock,
        Error::Deadline => Fault::Deadline,
        Error::WriteProgress => Fault::WriteProgress,
        Error::Line(_) => Fault::Line,
    }
}
