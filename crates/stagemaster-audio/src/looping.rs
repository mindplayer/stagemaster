//! Bounded PCM preparation off the audio callback and project lock.
use rodio::{ChannelCount, Decoder, SampleRate, Source};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};

pub const MAX_LOOP_MS: u64 = 60_000;
const MAX_SAMPLES: usize = 64 * 1024 * 1024 / size_of::<f32>();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoopRange {
    pub start_ms: u64,
    pub end_ms: u64,
}
impl LoopRange {
    /// # Errors
    /// Reject reversed, excessive, or out-of-clip ranges.
    pub fn validate(self, duration_ms: u64) -> Result<(), String> {
        if self.end_ms > duration_ms
            || self.end_ms.saturating_sub(self.start_ms) < 100
            || self.end_ms - self.start_ms > MAX_LOOP_MS
        {
            return Err("循环范围需在音乐内，长度为 0.100–60 秒".into());
        }
        Ok(())
    }
    pub(crate) fn contains(self, position: u64) -> bool {
        (self.start_ms..self.end_ms).contains(&position)
    }
}
pub struct LoopRequest {
    pub(crate) revision: u64,
    pub(crate) file: PathBuf,
    pub(crate) in_ms: u64,
    pub(crate) range: Option<LoopRange>,
}
pub struct PreparedLoop {
    pub(crate) revision: u64,
    pub(crate) file: PathBuf,
    pub(crate) in_ms: u64,
    pub(crate) buffer: Option<LoopBuffer>,
}
impl LoopRequest {
    /// Decode outside the transport lock; application must revalidate the revision before install.
    /// # Errors
    /// Reject inaccessible files, unsupported seeks, format, budgets, or incomplete PCM.
    pub fn prepare(self) -> Result<PreparedLoop, String> {
        let buffer = self
            .range
            .map(|range| {
                let file = File::open(&self.file).map_err(|e| format!("无法打开循环音乐：{e}"))?;
                let mut decoder =
                    Decoder::try_from(file).map_err(|e| format!("循环解码失败：{e}"))?;
                decoder
                    .try_seek(Duration::from_millis(self.in_ms + range.start_ms))
                    .map_err(|e| format!("循环定位失败：{e}"))?;
                LoopBuffer::decode(decoder, range)
            })
            .transpose()?;
        Ok(PreparedLoop {
            revision: self.revision,
            file: self.file,
            in_ms: self.in_ms,
            buffer,
        })
    }
}
pub(crate) struct LoopBuffer {
    range: LoopRange,
    samples: Arc<Vec<f32>>,
    channels: ChannelCount,
    rate: SampleRate,
}
impl LoopBuffer {
    fn decode(mut source: impl Source, range: LoopRange) -> Result<Self, String> {
        let channels = source.channels();
        let rate = source.sample_rate();
        let frames = (range.end_ms - range.start_ms) * u64::from(rate.get()) / 1000;
        let count =
            usize::try_from(frames * u64::from(channels.get())).map_err(|_| "循环样本过多")?;
        if channels.get() > 2 || count == 0 || count > MAX_SAMPLES {
            return Err("循环仅支持单／双声道，解码内存不得超过 64 MiB，请缩短范围".into());
        }
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(count)
            .map_err(|_| "循环内存不足，请缩短范围")?;
        for _ in 0..count {
            if source.channels() != channels || source.sample_rate() != rate {
                return Err("循环区间内的音频格式发生变化".into());
            }
            let value = source
                .next()
                .ok_or("循环范围超过实际音频，请缩短结束位置")?;
            if !value.is_finite() {
                return Err("循环音频包含无效样本".into());
            }
            samples.push(value);
        }
        Ok(Self {
            range,
            samples: Arc::new(samples),
            channels,
            rate,
        })
    }
    pub(crate) fn range(&self) -> LoopRange {
        self.range
    }
    fn frames(&self) -> u128 {
        (self.samples.len() / usize::from(self.channels.get())) as u128
    }
    fn offset(&self, base: u64) -> u128 {
        u128::from(base.saturating_sub(self.range.start_ms)) * u128::from(self.rate.get()) / 1000
    }
    pub(crate) fn position(&self, base: u64, elapsed: Duration) -> u64 {
        let frames = (self.offset(base)
            + elapsed.as_nanos() * u128::from(self.rate.get()) / 1_000_000_000)
            % self.frames();
        self.range.start_ms
            + u64::try_from(frames * 1000 / u128::from(self.rate.get())).unwrap_or(0)
    }
    pub(crate) fn source(&self, base: u64) -> LoopSource {
        LoopSource {
            samples: self.samples.clone(),
            cursor: usize::try_from(self.offset(base) % self.frames()).unwrap_or(0)
                * usize::from(self.channels.get()),
            channels: self.channels,
            rate: self.rate,
        }
    }
}
pub(crate) struct LoopSource {
    samples: Arc<Vec<f32>>,
    cursor: usize,
    channels: ChannelCount,
    rate: SampleRate,
}
impl Iterator for LoopSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let value = self.samples[self.cursor];
        self.cursor = (self.cursor + 1) % self.samples.len();
        Some(value)
    }
}
impl Source for LoopSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
#[path = "looping_tests.rs"]
mod tests;
