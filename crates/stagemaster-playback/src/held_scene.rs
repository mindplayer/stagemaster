//! Random-position sampling of a single held scene using the original renderer.
use crate::{MAX_TIME_MS, Plan, render};
use alloc::{string::String, vec::Vec};

pub struct HeldScene {
    plan: Plan,
    values: Vec<u16>,
}
impl HeldScene {
    /// Allocate only during preparation; retains the same interpolation and effect semantics as Player.
    /// # Errors
    /// Reject non-held plans or allocation failure.
    pub fn new(plan: Plan) -> Result<Self, String> {
        if plan.steps.len() != 1
            || plan.repeat
            || plan.steps[0].delay_ms != 0
            || plan.steps[0].wait_ms.is_some()
        {
            return Err("定点采样只支持无延时的单场景保持计划".into());
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(plan.defaults.len())
            .map_err(|_| "场景采样缓冲内存不足")?;
        values.extend_from_slice(&plan.defaults);
        Ok(Self { plan, values })
    }
    /// Sample in either direction without resetting a clock or allocating a new player.
    /// # Errors
    /// Reject out-of-range time before changing output.
    pub fn sample(&mut self, elapsed_ms: u64) -> Result<&[u16], String> {
        if elapsed_ms > MAX_TIME_MS {
            return Err("场景采样位置超出范围".into());
        }
        render::step(
            &self.plan,
            0,
            elapsed_ms,
            &self.plan.defaults,
            &mut self.values,
        );
        Ok(&self.values)
    }
    #[must_use]
    pub fn values(&self) -> &[u16] {
        &self.values
    }
}
