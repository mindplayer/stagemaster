use crate::{Command, Observer, Player, Status};
use alloc::string::String;

impl Player {
    /// Apply one command using the same execution path as the ordinary player API.
    /// Observer state must belong to this player and persist between commands.
    /// # Errors
    /// Invalid indices or backwards time leave player and observer unchanged.
    pub fn apply_observed(
        &mut self,
        command: Command,
        now_ms: u64,
        observer: &mut impl Observer,
    ) -> Result<(), String> {
        if let Command::Execute(index) = command
            && index >= self.plan.steps.len()
        {
            return Err("所选步骤不存在".into());
        }
        self.advance_with(now_ms, observer)?;
        match command {
            Command::Execute(index) => self.begin(index, now_ms, true, observer)?,
            Command::Next => {
                let next = self.index.map_or(0, |index| index + 1);
                if next < self.plan.steps.len() {
                    self.begin(next, now_ms, false, observer)?;
                } else if self.plan.repeat {
                    self.begin(0, now_ms, false, observer)?;
                }
            }
            Command::Pause if self.status == Status::Running => self.status = Status::Paused,
            Command::Resume if self.status == Status::Paused => self.status = Status::Running,
            Command::Stop => {
                self.status = Status::Idle;
                self.index = None;
                self.elapsed_ms = 0;
                self.activated = false;
                self.reassert = false;
                self.values.copy_from_slice(&self.plan.defaults);
                self.from.copy_from_slice(&self.plan.defaults);
                observer.stopped();
            }
            _ => {}
        }
        Ok(())
    }

    fn begin(
        &mut self,
        index: usize,
        now_ms: u64,
        reassert: bool,
        observer: &mut impl Observer,
    ) -> Result<(), String> {
        self.from.copy_from_slice(&self.values);
        self.index = Some(index);
        self.elapsed_ms = 0;
        self.status = Status::Running;
        self.activated = false;
        self.reassert = reassert;
        self.advance_with(now_ms, observer)
    }

    /// Advance to a monotonic timestamp without sleeps, allocations or frame catch-up.
    /// # Errors
    /// Backwards time leaves state unchanged.
    pub fn advance(&mut self, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Advance, now_ms, &mut ())
    }
    /// Execute a selected step from the current visible values; selection alone is external.
    /// # Errors
    /// Invalid step or backwards time does not mutate the player.
    pub fn execute(&mut self, index: usize, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Execute(index), now_ms, &mut ())
    }
    /// Advance manually, including interrupting the current fade.
    /// # Errors
    /// Rejects backwards time.
    pub fn next(&mut self, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Next, now_ms, &mut ())
    }
    #[must_use]
    pub fn can_next(&self) -> bool {
        self.index
            .is_none_or(|index| index + 1 < self.plan.steps.len() || self.plan.repeat)
    }
    /// # Errors
    /// Rejects backwards time.
    pub fn pause(&mut self, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Pause, now_ms, &mut ())
    }
    /// # Errors
    /// Rejects backwards time.
    pub fn resume(&mut self, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Resume, now_ms, &mut ())
    }
    /// Release the list contribution to profile defaults, not universally to zero.
    /// # Errors
    /// Rejects backwards time.
    pub fn stop(&mut self, now_ms: u64) -> Result<(), String> {
        self.apply_observed(Command::Stop, now_ms, &mut ())
    }
}
