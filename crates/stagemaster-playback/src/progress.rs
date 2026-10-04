//! Read-only presentation of the original player clock; never advances execution.
use crate::{Player, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Delay,
    Fade,
    Wait,
    Hold,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub phase: Phase,
    pub elapsed_ms: u64,
    pub phase_elapsed_ms: u64,
    pub phase_duration_ms: Option<u64>,
    pub next_step: Option<usize>,
    pub next_wrap: bool,
}

impl Player {
    /// Observe the current step without advancing or changing output.
    /// # Panics
    /// Only if the private active-state invariant is violated (missing step index).
    #[must_use]
    pub fn progress(&self) -> Progress {
        let mut result = Progress {
            phase: Phase::Idle,
            elapsed_ms: self.elapsed_ms,
            phase_elapsed_ms: 0,
            phase_duration_ms: None,
            next_step: None,
            next_wrap: false,
        };
        if self.status == Status::Idle {
            return result;
        }
        if self.status == Status::Finished {
            result.phase = Phase::Finished;
            return result;
        }
        let index = self.index.expect("active player has a step");
        let step = &self.plan.steps()[index];
        let fade_ms = step
            .fade_ms
            .saturating_sub(self.plan.entry_fade_offset_ms());
        if index + 1 < self.plan.steps().len() {
            result.next_step = Some(index + 1);
        } else if self.plan.repeat() {
            result.next_step = Some(0);
            result.next_wrap = true;
        }
        let (phase, offset, duration) = if self.elapsed_ms < step.delay_ms {
            (Phase::Delay, 0, Some(step.delay_ms))
        } else if self.elapsed_ms < step.delay_ms + fade_ms {
            (Phase::Fade, step.delay_ms, Some(fade_ms))
        } else {
            (
                if step.wait_ms.is_some() {
                    Phase::Wait
                } else {
                    Phase::Hold
                },
                step.delay_ms + fade_ms,
                step.wait_ms,
            )
        };
        result.phase = phase;
        result.phase_elapsed_ms = self.elapsed_ms - offset;
        result.phase_duration_ms = duration;
        result
    }
}
