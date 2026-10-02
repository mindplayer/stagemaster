use crate::{
    Code, Driver, EVENT_BUDGET, Event, Permit, Port, Ticket,
    port::{Flight, Owner},
};

impl<D: Driver> Port<D> {
    /// Service deadlines, at most `EVENT_BUDGET` reports, then at most one new frame.
    /// Continue polling after faults to observe actual shutdown; faults never clear.
    /// # Errors
    /// Return the latched clock, completion, driver or exhaustion fault.
    pub fn poll(&mut self, now_ms: u64) -> Result<(), Code> {
        let _ = self.advance(now_ms);
        for _ in 0..EVENT_BUDGET {
            match self.driver.poll() {
                Ok(Some(event)) => self.report(event),
                Ok(None) => break,
                Err(_) => {
                    self.fail(Code::Driver);
                    break;
                }
            }
        }
        if self.fault.is_none() && self.owner.is_some() && self.flight.is_none() {
            self.dispatch();
        }
        self.fault.map_or(Ok(()), Err)
    }

    fn dispatch(&mut self) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        if self.now >= pending.until {
            // Source expiry normally handles this first; do not start an old frame.
            self.reason = Some(Code::Stale);
            let _ = self.begin_quiescence(None);
            return;
        }
        let Some(deadline) = self.now.checked_add(self.config.ack_timeout_ms) else {
            self.fail(Code::Exhausted);
            return;
        };
        let ticket = match self.next_ticket(false) {
            Ok(ticket) => ticket,
            Err(code) => {
                self.fail(code);
                return;
            }
        };
        self.flight = Some(Flight {
            ticket,
            id: pending.id,
            deadline,
        });
        if self
            .driver
            .submit(ticket, &pending.slots, pending.until)
            .is_err()
        {
            self.fail(Code::Driver);
        } else {
            self.submitted = Some(pending.id);
        }
    }

    fn report(&mut self, event: Event) {
        let (Event::Sent(ticket) | Event::Expired(ticket) | Event::Quiet(ticket)) = event;
        if !self.issued(ticket) {
            self.fail(Code::DriverReport);
            return;
        }
        let is_stop = self.stopping.as_ref().is_some_and(|s| s.ticket == ticket);
        let is_frame = self.flight.as_ref().is_some_and(|f| f.ticket == ticket);
        match event {
            Event::Quiet(_) if is_stop => self.confirm_quiet(),
            Event::Sent(_) | Event::Expired(_) if is_frame => {
                let frame = self.flight.take().expect("matched flight");
                if matches!(event, Event::Sent(_)) {
                    self.completed = Some(frame.id);
                }
            }
            Event::Quiet(_) if is_frame => {
                self.fail(Code::DriverReport);
            }
            Event::Sent(_) | Event::Expired(_) if is_stop => {
                self.fail(Code::DriverReport);
            }
            // Old, already settled/cancelled receipts cannot affect a new operation.
            _ => {}
        }
    }

    fn issued(&self, ticket: Ticket) -> bool {
        ticket.boot == self.config.boot
            && ticket.port == self.config.port
            && ticket.counter > 0
            && ticket.counter <= self.counter
    }

    fn confirm_quiet(&mut self) {
        let stopping = self.stopping.take().expect("matched stop request");
        self.flight = None;
        self.pending = None;
        self.quiet = true;
        if self.fault.is_some() {
            return;
        }
        if let Some(source) = stopping.target {
            let Some(until) = self.now.checked_add(self.config.max_age_ms) else {
                self.fail(Code::Exhausted);
                return;
            };
            self.owner = Some(Owner {
                permit: Permit {
                    ticket: stopping.ticket,
                    source,
                },
                serial: 0,
                until,
            });
            self.quiet = false;
        }
    }
}
