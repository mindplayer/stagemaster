//! Deterministic, bounded single-list execution. The caller owns clocks and all I/O.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::{string::String, vec::Vec};
mod effect;
pub use effect::{Curve, EffectChannel, Keyframe, Transition};

pub const MAX_STEPS: usize = 1024;
pub const MAX_ATTRIBUTES: usize = 512;
pub const MAX_TARGET_VALUES: usize = 262_144;
pub const MAX_TIME_MS: u64 = 86_400_000;
pub const MAX_EFFECT_CHANNELS: usize = 16_384;
pub const MAX_KEYFRAMES: usize = 131_072;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub target: Vec<u16>,
    pub delay_ms: u64,
    pub fade_ms: u64,
    /// None waits for the operator; Some waits after the fade, then advances.
    pub wait_ms: Option<u64>,
}
impl Step {
    fn duration(&self) -> Option<u64> {
        self.wait_ms.map(|wait| self.delay_ms + self.fade_ms + wait)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    defaults: Vec<u16>,
    steps: Vec<Step>,
    repeat: bool,
    cycle_ms: Option<u64>,
    effects: Vec<Vec<EffectChannel>>,
}
impl Plan {
    /// Validate an in-memory plan. This is not a persistent/device wire format.
    /// # Errors
    /// Rejects dimensions, time or storage limits, and zero-duration automatic loops.
    pub fn new(defaults: Vec<u16>, steps: Vec<Step>, repeat: bool) -> Result<Self, String> {
        let effects = alloc::vec![Vec::new(); steps.len()];
        Self::with_effects(defaults, steps, repeat, effects)
    }
    /// Construct a bounded plan with one effect set per step.
    /// # Errors
    /// Rejects invalid or overlapping channels and oversized effect plans.
    pub fn with_effects(
        defaults: Vec<u16>,
        steps: Vec<Step>,
        repeat: bool,
        effects: Vec<Vec<EffectChannel>>,
    ) -> Result<Self, String> {
        if defaults.is_empty() || defaults.len() > MAX_ATTRIBUTES {
            return Err("预览需要 1–512 个灯具属性".into());
        }
        if steps.is_empty()
            || steps.len() > MAX_STEPS
            || steps.len() * defaults.len() > MAX_TARGET_VALUES
        {
            return Err("列表超出预览计划容量（1024 步、262144 个目标值）".into());
        }
        for step in &steps {
            if step.target.len() != defaults.len() {
                return Err("步骤目标与属性数量不一致".into());
            }
            if [Some(step.delay_ms), Some(step.fade_ms), step.wait_ms]
                .into_iter()
                .flatten()
                .any(|time| time > MAX_TIME_MS)
            {
                return Err("每项时间须在 0–86400 秒内".into());
            }
        }
        if effects.len() != steps.len()
            || effects.iter().map(Vec::len).sum::<usize>() > MAX_EFFECT_CHANNELS
        {
            return Err("效果计划超出容量或与步骤不一致".into());
        }
        let mut frame_count = 0;
        for channels in &effects {
            let mut occupied = [false; MAX_ATTRIBUTES];
            for channel in channels {
                if channel.index >= defaults.len()
                    || !(100..=3_600_000).contains(&channel.period_ms)
                    || !(1..=99).contains(&channel.duty_percent)
                {
                    return Err("效果属性、周期或亮段比例无效".into());
                }
                if occupied[channel.index] {
                    return Err("同一步骤的效果属性不能重叠".into());
                }
                occupied[channel.index] = true;
                if let Curve::Keyframes(frames) = &channel.curve {
                    frame_count += frames.len();
                    if !(2..=32).contains(&frames.len())
                        || frames[0].phase != 0
                        || frames.windows(2).any(|pair| pair[0].phase >= pair[1].phase)
                        || frame_count > MAX_KEYFRAMES
                    {
                        return Err("关键帧须从零开始递增，每条 2–32 帧且不得超过计划容量".into());
                    }
                }
            }
        }
        let cycle_ms = steps.iter().map(Step::duration).sum::<Option<u64>>();
        if repeat && cycle_ms == Some(0) {
            return Err("自动循环总时长不能为零，请增加渐变、延时或等待时间".into());
        }
        Ok(Self {
            defaults,
            steps,
            repeat,
            cycle_ms,
            effects,
        })
    }
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
    #[must_use]
    pub fn defaults(&self) -> &[u16] {
        &self.defaults
    }
    #[must_use]
    pub fn effect_channel_count(&self) -> usize {
        self.effects.iter().map(Vec::len).sum()
    }
    #[must_use]
    pub fn keyframe_count(&self) -> usize {
        self.effects
            .iter()
            .flatten()
            .map(|channel| match &channel.curve {
                Curve::Keyframes(frames) => frames.len(),
                _ => 0,
            })
            .sum()
    }
    /// Exact bytes of u16 target/default/current/start buffers, excluding metadata/allocator overhead.
    #[must_use]
    pub fn value_buffer_bytes(&self) -> usize {
        (self.steps.len() + 3) * self.defaults.len() * 2
    }
    /// Effect payload and inline curve metadata, excluding outer Vec headers and allocator overhead.
    #[must_use]
    pub fn effect_buffer_bytes(&self) -> usize {
        self.effects
            .iter()
            .flatten()
            .map(|channel| {
                core::mem::size_of::<EffectChannel>()
                    + match &channel.curve {
                        Curve::Keyframes(frames) => frames.len() * core::mem::size_of::<Keyframe>(),
                        _ => 0,
                    }
            })
            .sum()
    }
}
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
            self.values[effect.index] = effect.sample(elapsed);
        }
        if elapsed < step.fade_ms {
            for (value, from) in self.values.iter_mut().zip(&self.from) {
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
