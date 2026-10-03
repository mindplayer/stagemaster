//! Board supervision policy. Hardware reset and clocks stay in the platform adapter.
pub const STARTUP_MS: u64 = 10_000;
pub const PROGRESS_MS: u64 = 2_000;
pub const FAILED: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Worker,
    Clock,
    Deadline,
    Sequence,
    SenderExited,
}

pub struct Policy {
    observed: u64,
    deadline: u64,
    sequence: u32,
    fault: Option<Fault>,
}
impl Policy {
    pub fn new(now: u64) -> Self {
        Self {
            observed: now,
            deadline: now.saturating_add(STARTUP_MS),
            sequence: 0,
            fault: now
                .checked_add(STARTUP_MS)
                .is_none()
                .then_some(Fault::Clock),
        }
    }

    /// A late new heartbeat cannot retroactively revive an expired supervisor.
    pub fn check(&mut self, now: u64, sequence: u32) -> Result<(), Fault> {
        if let Some(fault) = self.fault {
            return Err(fault);
        }
        let fault = if now < self.observed {
            Some(Fault::Clock)
        } else if sequence == FAILED {
            Some(Fault::Worker)
        } else if sequence < self.sequence {
            Some(Fault::Sequence)
        } else if now >= self.deadline {
            Some(Fault::Deadline)
        } else {
            None
        };
        if let Some(fault) = fault {
            return self.trip(fault);
        }
        self.observed = now;
        if sequence > self.sequence {
            let Some(deadline) = now.checked_add(PROGRESS_MS) else {
                return self.trip(Fault::Clock);
            };
            self.sequence = sequence;
            self.deadline = deadline;
        }
        Ok(())
    }

    pub fn trip(&mut self, fault: Fault) -> Result<(), Fault> {
        Err(*self.fault.get_or_insert(fault))
    }
}
