use super::{
    BREAK_BITS, Clock, Error, FRAME_BYTES, Gate, Line, MARK_US, Transmitter,
    deadline::{bounded, observe},
};
use crate::{Event, Ticket};

impl<L: Line, C: Clock> Transmitter<L, C> {
    /// One complete frame, never an implicit repeat. Cancellation gates off RS485;
    /// the next operation must still drain any UART bytes left behind.
    /// # Errors
    /// Reports clock, timeout, invalid write progress or actual line failure.
    pub async fn send(
        &mut self,
        ticket: Ticket,
        slots: &[u8; 512],
        valid_until_ms: u64,
    ) -> Result<Event, Error<L::Error>> {
        let mut gate = Gate::new(&mut self.line);
        let start = observe(&self.clock, &mut self.last_us)?;
        let deadline = start.checked_add(self.timeout_us).ok_or(Error::Exhausted)?;
        if start / 1000 >= valid_until_ms {
            return Ok(Event::Expired(ticket));
        }
        // A cancelled previous frame can still be in the UART, but its gate is off.
        bounded(&self.clock, &mut self.last_us, deadline, gate.line.drain()).await?;
        let mut packet = [0; FRAME_BYTES];
        packet[1..].copy_from_slice(slots);
        let now = observe(&self.clock, &mut self.last_us)?;
        if now / 1000 >= valid_until_ms {
            return Ok(Event::Expired(ticket));
        }
        if now >= deadline {
            return Err(Error::Deadline);
        }
        gate.line.enable();
        bounded(
            &self.clock,
            &mut self.last_us,
            deadline,
            gate.line.break_signal(BREAK_BITS),
        )
        .await?;
        let mark_until = observe(&self.clock, &mut self.last_us)?
            .checked_add(MARK_US)
            .ok_or(Error::Exhausted)?;
        bounded(&self.clock, &mut self.last_us, deadline, async {
            self.clock.wait_until_us(mark_until).await;
            Ok(())
        })
        .await?;
        if observe(&self.clock, &mut self.last_us)? < mark_until {
            return Err(Error::Clock);
        }
        let mut offset = 0;
        while offset < packet.len() {
            let count = bounded(
                &self.clock,
                &mut self.last_us,
                deadline,
                gate.line.write(&packet[offset..]),
            )
            .await?;
            if count == 0 || count > packet.len() - offset {
                return Err(Error::WriteProgress);
            }
            offset += count;
        }
        bounded(&self.clock, &mut self.last_us, deadline, gate.line.drain()).await?;
        // Keep MARK on the bus between successful packets. Drop/error paths gate off.
        gate.armed = false;
        Ok(Event::Sent(ticket))
    }

    /// Disable first, then confirm that no cancelled bytes or shift operation remain.
    /// # Errors
    /// Failure is never a Quiet acknowledgement, even though the gate is off.
    pub async fn quiesce(&mut self, ticket: Ticket) -> Result<Event, Error<L::Error>> {
        let gate = Gate::new(&mut self.line);
        gate.line.disable();
        let now = observe(&self.clock, &mut self.last_us)?;
        let deadline = now.checked_add(self.timeout_us).ok_or(Error::Exhausted)?;
        bounded(&self.clock, &mut self.last_us, deadline, gate.line.drain()).await?;
        Ok(Event::Quiet(ticket))
    }
}
