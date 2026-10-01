//! Deterministic, bounded single-list execution. The caller owns clocks and all I/O.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::{string::String, vec::Vec};
mod effect;
mod output_master;
mod plan;
mod rate_clock;
pub use effect::{Curve, EffectChannel, Keyframe, Transition};
pub use output_master::OutputMaster;
pub use plan::{Plan, Step};
pub use rate_clock::RateClock;

pub const MAX_STEPS: usize = 1024;
pub const MAX_ATTRIBUTES: usize = 512;
pub const MAX_TARGET_VALUES: usize = 262_144;
pub const MAX_TIME_MS: u64 = 86_400_000;
pub const MAX_EFFECT_CHANNELS: usize = 16_384;
pub const MAX_KEYFRAMES: usize = 131_072;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Paused,
    Finished,
}

pub struct Player {
    plan: Plan,
    status: Status,
    index: Option<usize>,
    elapsed_ms: u64,
    last_ms: u64,
    values: Vec<u16>,
    from: Vec<u16>,
}
impl Player {
    /// Construct with recoverable allocation failures for memory-constrained hosts.
    /// # Errors
    /// Return the allocator error if either live value buffer cannot be reserved.
    pub fn try_new(plan: Plan, now_ms: u64) -> Result<Self, alloc::collections::TryReserveError> {
        let mut values = Vec::new();
        values.try_reserve_exact(plan.defaults.len())?;
        values.extend_from_slice(&plan.defaults);
        let mut from = Vec::new();
        from.try_reserve_exact(plan.defaults.len())?;
        from.extend_from_slice(&plan.defaults);
        Ok(Self {
            plan,
            status: Status::Idle,
            index: None,
            elapsed_ms: 0,
            last_ms: now_ms,
            values,
            from,
        })
    }

    #[must_use]
    pub fn new(plan: Plan, now_ms: u64) -> Self {
        Self {
            values: plan.defaults.clone(),
            from: plan.defaults.clone(),
            plan,
            status: Status::Idle,
            index: None,
            elapsed_ms: 0,
            last_ms: now_ms,
        }
    }
    #[must_use]
    pub fn values(&self) -> &[u16] {
        &self.values
    }
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        self.index
    }
    #[must_use]
    pub const fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }
    #[must_use]
    pub const fn plan(&self) -> &Plan {
        &self.plan
    }

    /// Advance once to a monotonic timestamp. No sleeps, allocations or frame catch-up.
    /// # Errors
    /// A backwards timestamp leaves all state unchanged.
    pub fn advance(&mut self, now_ms: u64) -> Result<(), String> {
        let delta = now_ms.checked_sub(self.last_ms).ok_or("播放时钟不能倒退")?;
        self.last_ms = now_ms;
        if self.status != Status::Running {
            return Ok(());
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(delta);
        loop {
            let Some(index) = self.index else {
                return Err("播放状态缺少活动步骤".into());
            };
            let step = &self.plan.steps[index];
            let Some(duration) = step.duration() else {
                break;
            };
            if self.elapsed_ms < duration {
                break;
            }
            let remaining = self.elapsed_ms - duration;
            self.elapsed_ms = duration;
            self.render();
            self.elapsed_ms = remaining;
            if index + 1 == self.plan.steps.len() {
                if !self.plan.repeat {
                    self.status = Status::Finished;
                    self.elapsed_ms = duration;
                    return Ok(());
                }
                // At a cycle boundary the prior target is known, even after an interrupted fade.
                if let Some(cycle) = self.plan.cycle_ms {
                    self.elapsed_ms %= cycle;
                }
                self.index = Some(0);
            } else {
                self.index = Some(index + 1);
            }
            self.from.copy_from_slice(&self.values);
        }
        self.render();
        Ok(())
    }
    fn render(&mut self) {
        let index = self.index.expect("active step");
        let step = &self.plan.steps[index];
        if self.elapsed_ms < step.delay_ms {
            self.values.copy_from_slice(&self.from);
            return;
        }
        let elapsed = self.elapsed_ms - step.delay_ms;
        self.values.copy_from_slice(&step.target);
        for effect in &self.plan.effects[index] {
            self.values[effect.index] = effect.sample(elapsed, self.plan.effect_time_offset_ms);
        }
        if elapsed < step.fade_ms {
            let mut snap = self.plan.snap_attributes.iter().peekable();
            for (index, (value, from)) in self.values.iter_mut().zip(&self.from).enumerate() {
                if snap.peek().is_some_and(|&&i| usize::from(i) == index) {
                    snap.next();
                    continue;
                }
                let weighted =
                    u64::from(*from) * (step.fade_ms - elapsed) + u64::from(*value) * elapsed;
                *value = u16::try_from((weighted + step.fade_ms / 2) / step.fade_ms)
                    .expect("bounded interpolation");
            }
        }
    }
    /// Execute a selected step from the current visible values; selection alone is external.
    /// # Errors
    /// Invalid step or backwards time does not mutate the player.
    pub fn execute(&mut self, index: usize, now_ms: u64) -> Result<(), String> {
        if index >= self.plan.steps.len() {
            return Err("所选步骤不存在".into());
        }
        self.advance(now_ms)?;
        self.from.copy_from_slice(&self.values);
        self.index = Some(index);
        self.elapsed_ms = 0;
        self.status = Status::Running;
        self.advance(now_ms)
    }
    /// Advance manually, including interrupting the current fade.
    /// # Errors
    /// Rejects backwards time.
    pub fn next(&mut self, now_ms: u64) -> Result<(), String> {
        self.advance(now_ms)?;
        let next = self.index.map_or(0, |index| index + 1);
        if next < self.plan.steps.len() {
            self.execute(next, now_ms)
        } else if self.plan.repeat {
            self.execute(0, now_ms)
        } else {
            // No following step: leave a still-running fade and manual hold intact.
            Ok(())
        }
    }
    #[must_use]
    pub fn can_next(&self) -> bool {
        self.index
            .is_none_or(|index| index + 1 < self.plan.steps.len() || self.plan.repeat)
    }
    /// # Errors
    /// Rejects backwards time.
    pub fn pause(&mut self, now_ms: u64) -> Result<(), String> {
        self.advance(now_ms)?;
        if self.status == Status::Running {
            self.status = Status::Paused;
        }
        Ok(())
    }
    /// # Errors
    /// Rejects backwards time.
    pub fn resume(&mut self, now_ms: u64) -> Result<(), String> {
        self.advance(now_ms)?;
        if self.status == Status::Paused {
            self.status = Status::Running;
        }
        Ok(())
    }
    /// Release the list contribution to its profile defaults, not universally to zero.
    /// # Errors
    /// Rejects backwards time.
    pub fn stop(&mut self, now_ms: u64) -> Result<(), String> {
        self.advance(now_ms)?;
        self.status = Status::Idle;
        self.index = None;
        self.elapsed_ms = 0;
        self.values.copy_from_slice(&self.plan.defaults);
        self.from.copy_from_slice(&self.plan.defaults);
        Ok(())
    }
}
