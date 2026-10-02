use crate::Ticket;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The exact frame has completed transmission, not merely entered a queue.
    Sent(Ticket),
    /// Accepted frame was discarded because it expired before transmission started.
    Expired(Ticket),
    /// No queued, in-flight or autonomous repeated transmission remains.
    Quiet(Ticket),
}
/// Exclusively owned, bounded, nonblocking adapter. No hidden frame backlog.
///
/// At most one frame may be in flight. Methods must return promptly; the host must
/// poll frequently. The driver must independently enforce expiry/watchdog policy if
/// the host stalls. Successful method return means acceptance, never completion.
/// Errors may leave hardware state uncertain; quiescence must still be attempted.
/// Dropping the port is not proof of physical shutdown.
pub trait Driver {
    type Error;

    /// Copy/own the complete frame before returning; no implicit repeated sending.
    /// Check `valid_until_ms` against the same monotonic clock at actual start.
    /// Return Sent only after completion, or Expired if no transmission began.
    /// # Errors
    /// Report admission/hardware failure without claiming the output is quiet.
    fn submit(
        &mut self,
        ticket: Ticket,
        slots: &[u8; 512],
        valid_until_ms: u64,
    ) -> Result<(), Self::Error>;

    /// Disable future sends, discard queued work, finish/abort any in-flight work.
    /// Emit the exact Quiet ticket only once this is actually true.
    /// # Errors
    /// Report failure; an error never counts as a Quiet acknowledgement.
    fn quiesce(&mut self, ticket: Ticket) -> Result<(), Self::Error>;

    /// Return at most one completion. The adapter itself must bound its event queue.
    /// # Errors
    /// Report driver fault, including completion queue overflow.
    fn poll(&mut self) -> Result<Option<Event>, Self::Error>;
}
