use crate::{Code, Config, Driver, FrameId, Permit, Phase, Sample, Source, State, Ticket};

pub(crate) struct Owner {
    pub permit: Permit,
    pub serial: u64,
    pub until: u64,
}
pub(crate) struct Pending {
    pub id: FrameId,
    pub slots: [u8; 512],
    pub until: u64,
}
pub(crate) struct Flight {
    pub ticket: Ticket,
    pub id: FrameId,
    pub deadline: u64,
}
pub(crate) struct Stopping {
    pub ticket: Ticket,
    pub deadline: u64,
    pub target: Option<Source>,
}
/// One output port, one driver, one current source. Contains no heap allocation.
/// New instances start unconfirmed; even initial ownership needs a Quiet receipt.
pub struct Port<D: Driver> {
    pub(crate) driver: D,
    pub(crate) config: Config,
    pub(crate) now: u64,
    pub(crate) counter: u64,
    pub(crate) owner: Option<Owner>,
    pub(crate) pending: Option<Pending>,
    pub(crate) flight: Option<Flight>,
    pub(crate) stopping: Option<Stopping>,
    pub(crate) quiet: bool,
    pub(crate) fault: Option<Code>,
    pub(crate) reason: Option<Code>,
    pub(crate) accepted: Option<FrameId>,
    pub(crate) submitted: Option<FrameId>,
    pub(crate) completed: Option<FrameId>,
}
impl<D: Driver> Port<D> {
    /// # Errors
    /// Reject zero identity/port/universe, invalid budgets, or exhausted clock.
    pub fn new(config: Config, driver: D, now_ms: u64) -> Result<Self, Code> {
        if config.boot == [0; 16]
            || config.port == 0
            || config.universe == 0
            || !(1..=60_000).contains(&config.max_age_ms)
            || !(1..=60_000).contains(&config.ack_timeout_ms)
        {
            return Err(Code::Identity);
        }
        now_ms
            .checked_add(config.max_age_ms.max(config.ack_timeout_ms))
            .ok_or(Code::Exhausted)?;
        Ok(Self {
            driver,
            config,
            now: now_ms,
            counter: 0,
            owner: None,
            pending: None,
            flight: None,
            stopping: None,
            quiet: false,
            fault: None,
            reason: None,
            accepted: None,
            submitted: None,
            completed: None,
        })
    }

    #[must_use]
    pub fn state(&self) -> State {
        let phase = if self.fault.is_some() {
            Phase::Faulted
        } else if self.stopping.is_some() {
            Phase::Quiescing
        } else if self.owner.is_some() {
            Phase::Ready
        } else if self.quiet {
            Phase::Idle
        } else {
            Phase::Unconfirmed
        };
        State {
            phase,
            permit: self.owner.as_ref().map(|o| o.permit),
            quiet: self.quiet,
            stopping: self.stopping.as_ref().map(|s| s.ticket),
            pending: self.pending.as_ref().map(|p| p.id),
            in_flight: self.flight.as_ref().map(|f| f.id),
            accepted: self.accepted,
            submitted: self.submitted,
            completed: self.completed,
            reason: self.fault.or(self.reason),
        }
    }

    /// Trusted host selection; `takeover` is explicit, not remote authentication.
    /// A returned ticket is a stop request, not a usable output permit.
    /// # Errors
    /// Reject invalid identity, occupied/quiescing port, clock or terminal fault.
    pub fn select(&mut self, source: Source, takeover: bool, now_ms: u64) -> Result<Ticket, Code> {
        self.advance(now_ms)?;
        if source.id == [0; 16] {
            return Err(Code::Identity);
        }
        if self.stopping.is_some() || (self.owner.is_some() && !takeover) {
            return Err(Code::Busy);
        }
        self.reason = None;
        self.begin_quiescence(Some(source))
    }

    /// Replace the pending frame with the latest complete sample.
    /// Frame dispatch happens in poll; deadline/fault handling may request quiescence.
    /// # Errors
    /// Reject stale authority, serial, universe, future/old samples, clock or fault.
    pub fn submit(&mut self, permit: Permit, sample: Sample<'_>, now_ms: u64) -> Result<(), Code> {
        self.advance(now_ms)?;
        let owner = self.owner.as_ref().ok_or(Code::Permit)?;
        if owner.permit != permit {
            return Err(Code::Permit);
        }
        if sample.serial == 0 || sample.serial <= owner.serial {
            return Err(Code::Sequence);
        }
        if sample.universe != self.config.universe {
            return Err(Code::Universe);
        }
        if sample.sampled_ms > now_ms {
            return Err(Code::Future);
        }
        let until = sample.sampled_ms.checked_add(self.config.max_age_ms);
        let Some(until) = until else {
            return Err(self.fail(Code::Exhausted));
        };
        if self.accepted.is_some_and(|previous| {
            previous.permit == permit && sample.sampled_ms < previous.sampled_ms
        }) || now_ms >= until
        {
            return Err(Code::Stale);
        }
        let id = FrameId {
            permit,
            serial: sample.serial,
            sampled_ms: sample.sampled_ms,
        };
        self.owner = Some(Owner {
            permit,
            serial: sample.serial,
            until,
        });
        self.pending = Some(Pending {
            id,
            slots: *sample.slots,
            until,
        });
        self.accepted = Some(id);
        Ok(())
    }

    /// Stop only if the caller still owns this port. Does not send an all-zero frame.
    /// # Errors
    /// Reject old permits, invalid time and terminal faults.
    pub fn stop(&mut self, permit: Permit, now_ms: u64) -> Result<Ticket, Code> {
        self.advance(now_ms)?;
        if self.owner.as_ref().map(|o| o.permit) != Some(permit) {
            return Err(Code::Permit);
        }
        self.begin_quiescence(None)
    }

    /// Trusted host shutdown also cancels a pending takeover. Poll until confirmed.
    /// # Errors
    /// Report clock/driver/exhaustion faults while still attempting shutdown.
    pub fn shutdown(&mut self, now_ms: u64) -> Result<Option<Ticket>, Code> {
        let result = self.advance(now_ms);
        if let Some(stopping) = &mut self.stopping {
            stopping.target = None;
            return result.map(|()| Some(stopping.ticket));
        }
        if self.quiet {
            return result.map(|()| None);
        }
        let ticket = self.begin_quiescence(None)?;
        result.map(|()| Some(ticket))
    }

    /// Run synchronous maintenance only in a currently confirmed quiet window.
    /// No proof is returned for later reuse; the closure must not return a Future.
    /// # Errors
    /// Reject active, unconfirmed, quiescing or faulted ports.
    pub fn with_quiescent(&mut self, work: impl FnOnce()) -> Result<(), Code> {
        if self.fault.is_some() || !self.quiet || self.owner.is_some() || self.stopping.is_some() {
            return Err(Code::NotQuiet);
        }
        work();
        Ok(())
    }
}
