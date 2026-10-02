//! Two held scenes sampled from caller-owned time. No second clock or recursive plan graph.
use crate::{MAX_TIME_MS, Plan, render};
use alloc::{string::String, vec::Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossfadeTiming {
    pub duration_ms: u64,
    /// Progress in the original transition at this visible range's start.
    pub offset_ms: u64,
    /// Local source scene time at this range's start, independent of its effect offset.
    pub source_elapsed_ms: u64,
    pub target_elapsed_ms: u64,
}

pub struct SceneCrossfade {
    source: Plan,
    target: Plan,
    timing: CrossfadeTiming,
    source_values: Vec<u16>,
    values: Vec<u16>,
}

impl SceneCrossfade {
    /// Both plans must use the same ordered semantic attribute bindings (verified by their compiler).
    /// Allocation occurs only here; successful samples reuse the two bounded value buffers.
    /// # Errors
    /// Rejects non-held plans, mismatched dimensions/discrete masks, invalid clocks or allocation failure.
    pub fn new(source: Plan, target: Plan, timing: CrossfadeTiming) -> Result<Self, String> {
        for plan in [&source, &target] {
            if plan.steps.len() != 1
                || plan.repeat
                || plan.steps[0].delay_ms != 0
                || plan.steps[0].wait_ms.is_some()
            {
                return Err("动态交叉仅支持两个无延时的单场景保持计划".into());
            }
        }
        if source.defaults.len() != target.defaults.len()
            || source.snap_attributes != target.snap_attributes
        {
            return Err("动态交叉的属性数量与直接切换映射必须一致".into());
        }
        if timing.duration_ms == 0 || timing.duration_ms > MAX_TIME_MS {
            return Err("动态交叉时长须大于零且不超过 86400 秒".into());
        }
        let mut sampler = Self {
            source_values: buffer(source.defaults.len())?,
            values: buffer(target.defaults.len())?,
            source,
            target,
            timing,
        };
        sampler.sample(0)?;
        Ok(sampler)
    }

    /// Sample from this range's start in any order. The result does not depend on prior samples.
    /// # Errors
    /// All three timeline positions must stay within 24 hours; failure leaves prior output intact.
    pub fn sample(&mut self, elapsed_ms: u64) -> Result<&[u16], String> {
        let [source_time, target_time, progress] = [
            self.timing.source_elapsed_ms,
            self.timing.target_elapsed_ms,
            self.timing.offset_ms,
        ]
        .map(|start| {
            start
                .checked_add(elapsed_ms)
                .filter(|&time| time <= MAX_TIME_MS)
        });
        let (Some(source_time), Some(target_time), Some(progress)) =
            (source_time, target_time, progress)
        else {
            return Err("动态交叉的源时间或渐变时间超出 86400 秒".into());
        };
        render::step(
            &self.source,
            0,
            source_time,
            &self.source.defaults,
            &mut self.source_values,
        );
        render::step(
            &self.target,
            0,
            target_time,
            &self.target.defaults,
            &mut self.values,
        );
        render::blend(
            &self.source_values,
            &mut self.values,
            &self.target.snap_attributes,
            progress,
            self.timing.duration_ms,
        );
        Ok(&self.values)
    }

    #[must_use]
    pub fn values(&self) -> &[u16] {
        &self.values
    }
}

fn buffer(len: usize) -> Result<Vec<u16>, String> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(len)
        .map_err(|_| String::from("动态交叉输出缓冲内存不足"))?;
    result.resize(len, 0);
    Ok(result)
}
