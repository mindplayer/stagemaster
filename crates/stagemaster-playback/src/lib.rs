//! Deterministic, bounded lighting execution and scene sampling. The caller owns clocks and all I/O.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::vec::Vec;
mod advance;
mod commands;
mod crossfade;
mod effect;
mod loop_schedule;
mod observation;
mod output_master;
mod plan;
mod rate_clock;
mod render;
pub use crossfade::{CrossfadeTiming, SceneCrossfade};
pub use effect::{Curve, EffectChannel, Keyframe, Transition};
pub use loop_schedule::{
    LoopPlayback, LoopPlays, LoopPosition, LoopRegion, LoopSchedule, MAX_LOOP_REGIONS,
};
pub use observation::{Command, Observer};
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
    activated: bool,
    reassert: bool,
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
            activated: false,
            reassert: false,
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
            activated: false,
            reassert: false,
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
}
