use crate::{Code, Driver, Port, Source, Ticket, port::Stopping};

impl<D: Driver> Port<D> {
    pub(crate) fn advance(&mut self, now_ms: u64) -> Result<(), Code> {
        if now_ms < self.now {
            return Err(self.fail(Code::Clock));
        }
        self.now = now_ms;
        let missed = self.stopping.as_ref().map_or_else(
            || self.flight.as_ref().is_some_and(|f| now_ms >= f.deadline),
            |s| now_ms >= s.deadline,
        );
        if missed {
            self.fail(Code::Deadline);
        }
        if let Some(fault) = self.fault {
            return Err(fault);
        }
        if self.owner.as_ref().is_some_and(|o| now_ms >= o.until) {
            self.reason = Some(Code::Stale);
            self.begin_quiescence(None)?;
        }
        Ok(())
    }

    pub(crate) fn next_ticket(&mut self, stopping: bool) -> Result<Ticket, Code> {
        // Reserve the final unique value for one last shutdown, never for a frame.
        let counter = self.counter.checked_add(1).ok_or(Code::Exhausted)?;
        if !stopping && counter == u64::MAX {
            return Err(Code::Exhausted);
        }
        self.counter = counter;
        Ok(Ticket {
            boot: self.config.boot,
            port: self.config.port,
            counter,
        })
    }

    pub(crate) fn begin_quiescence(&mut self, target: Option<Source>) -> Result<Ticket, Code> {
        self.owner = None;
        self.pending = None;
        self.quiet = false;
        let ticket = self.next_ticket(true).inspect_err(|error| {
            self.fault.get_or_insert(*error);
        })?;
        let deadline = self.now.checked_add(self.config.ack_timeout_ms);
        if deadline.is_none() || ticket.counter == u64::MAX {
            self.fault.get_or_insert(Code::Exhausted);
        }
        self.stopping = Some(Stopping {
            ticket,
            deadline: deadline.unwrap_or(u64::MAX),
            target: if self.fault.is_some() { None } else { target },
        });
        // Keep the request even if admission failed: a late genuine Quiet can still
        // establish physical quiet, but can never clear a terminal software fault.
        if self.driver.quiesce(ticket).is_err() {
            self.fault.get_or_insert(Code::Driver);
            self.stopping.as_mut().expect("request exists").target = None;
        }
        self.fault.map_or(Ok(ticket), Err)
    }

    pub(crate) fn fail(&mut self, code: Code) -> Code {
        let fault = *self.fault.get_or_insert(code);
        self.owner = None;
        self.pending = None;
        if let Some(stopping) = &mut self.stopping {
            stopping.target = None;
        } else if !self.quiet {
            let _ = self.begin_quiescence(None);
        }
        fault
    }
}
