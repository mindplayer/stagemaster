use stagemaster_playback::{MAX_EFFECT_CHANNELS, MAX_KEYFRAMES, MAX_TARGET_VALUES, Plan};

#[derive(Clone, Copy, Default)]
pub struct LiveSourceBudget {
    pub targets: usize,
    pub effect_channels: usize,
    pub keyframes: usize,
}
impl LiveSourceBudget {
    #[must_use]
    pub fn from_plan(plan: &Plan) -> Self {
        Self {
            targets: plan.steps().len() * plan.defaults().len(),
            effect_channels: plan.effect_channel_count(),
            keyframes: plan.keyframe_count(),
        }
    }
    /// Charge all resident plans, including both sides of a dynamic transition.
    /// # Errors
    /// Reject overflow or exceeding the existing shared host budget, without changing the counter.
    pub fn add(&mut self, other: Self) -> Result<(), String> {
        let add = |a: usize, b: usize, max: usize| {
            a.checked_add(b)
                .filter(|v| *v <= max)
                .ok_or("来源组累计目标值、效果或关键帧超出宿主预算")
        };
        let next = Self {
            targets: add(self.targets, other.targets, MAX_TARGET_VALUES)?,
            effect_channels: add(
                self.effect_channels,
                other.effect_channels,
                MAX_EFFECT_CHANNELS,
            )?,
            keyframes: add(self.keyframes, other.keyframes, MAX_KEYFRAMES)?,
        };
        *self = next;
        Ok(())
    }
}
