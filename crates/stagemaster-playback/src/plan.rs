//! Validated immutable playback plans. No clocks, hardware or project parsing.
use crate::{
    Curve, EffectChannel, Keyframe, MAX_ATTRIBUTES, MAX_EFFECT_CHANNELS, MAX_KEYFRAMES, MAX_STEPS,
    MAX_TARGET_VALUES, MAX_TIME_MS,
};
use alloc::{string::String, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub target: Vec<u16>,
    pub delay_ms: u64,
    pub fade_ms: u64,
    /// None waits for the operator; Some waits after the fade, then advances.
    pub wait_ms: Option<u64>,
}
impl Step {
    pub(super) fn duration(&self) -> Option<u64> {
        self.wait_ms.map(|wait| self.delay_ms + self.fade_ms + wait)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub(super) defaults: Vec<u16>,
    pub(super) steps: Vec<Step>,
    pub(super) repeat: bool,
    pub(super) cycle_ms: Option<u64>,
    pub(super) effects: Vec<Vec<EffectChannel>>,
    pub(super) snap_attributes: Vec<u16>,
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
        Self::with_snap_attributes(defaults, steps, repeat, effects, Vec::new())
    }
    /// Construct a plan with attributes that switch immediately after each step's delay.
    /// Indices must be strictly increasing. Snap attributes cannot own dynamic effects.
    /// # Errors
    /// Rejects invalid indices, effect conflicts, and all ordinary plan limits.
    pub fn with_snap_attributes(
        defaults: Vec<u16>,
        steps: Vec<Step>,
        repeat: bool,
        effects: Vec<Vec<EffectChannel>>,
        snap_attributes: Vec<u16>,
    ) -> Result<Self, String> {
        if snap_attributes
            .iter()
            .any(|&i| usize::from(i) >= defaults.len())
            || snap_attributes.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err("直接切换属性须在计划范围内、递增且不重复".into());
        }
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
                if snap_attributes
                    .binary_search_by_key(&channel.index, |&i| usize::from(i))
                    .is_ok()
                {
                    return Err("直接切换属性不能使用连续动态效果".into());
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
            snap_attributes,
        })
    }
    #[must_use]
    pub fn snap_attributes(&self) -> &[u16] {
        &self.snap_attributes
    }
    /// Payload bytes for discrete attribute indices; empty plans allocate no index buffer.
    #[must_use]
    pub fn snap_buffer_bytes(&self) -> usize {
        self.snap_attributes.len() * core::mem::size_of::<u16>()
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
    pub const fn repeat(&self) -> bool {
        self.repeat
    }
    #[must_use]
    pub fn effects(&self) -> &[Vec<EffectChannel>] {
        &self.effects
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
