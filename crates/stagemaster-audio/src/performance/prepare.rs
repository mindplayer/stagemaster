use super::{
    PerformanceControl, PerformanceSource,
    decode::{self, Format},
};
use stagemaster_playback::{LoopPlays, LoopRegion, LoopSchedule};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

pub const MAX_PERFORMANCE_CACHE_BYTES: usize = 64 * 1_024 * 1_024;
type CachedRegions = Vec<Option<Vec<f32>>>;

pub(super) struct Data {
    pub path: PathBuf,
    pub format: Format,
    pub source_start: u64,
    pub duration_ms: u64,
    pub schedule: LoopSchedule,
    pub cached: CachedRegions,
    pub cache_bytes: usize,
}

/// Immutable prepared repeat ranges; ordinary source spans remain streamed.
#[derive(Clone)]
pub struct PerformanceAudio {
    pub(super) data: Arc<Data>,
}

impl PerformanceAudio {
    /// Prepare off the project lock. The input schedule uses local milliseconds.
    /// The host must supply an integrity-checked, immutable media resource for this lifetime.
    /// # Errors
    /// Reject unsupported media, cancelled/expired work, short data or excess aggregate cache.
    pub fn prepare(
        path: PathBuf,
        in_ms: u64,
        schedule: &LoopSchedule,
        cancel: &AtomicBool,
    ) -> Result<Self, String> {
        let started = Instant::now();
        decode::checkpoint(cancel, started)?;
        if in_ms
            .checked_add(schedule.duration())
            .is_none_or(|v| v > crate::MAX_DURATION_MS)
        {
            return Err("演出音乐范围超过允许长度".into());
        }
        let mut decoder = decode::open(&path)?;
        let format = Format::read(&decoder)?;
        let duration_ms = schedule.duration();
        let regions = schedule
            .regions()
            .iter()
            .map(|region| {
                Ok(LoopRegion {
                    start: format.frames(region.start)?,
                    end: format.frames(region.end)?,
                    plays: region.plays,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let schedule = LoopSchedule::new(format.frames(duration_ms)?, regions)?;
        let source_start = format.frames(in_ms)?;
        let (mut cached, cache_bytes) = allocate(&schedule, format)?;
        let mut frame_index = 0;
        for (region, samples) in schedule.regions().iter().zip(&mut cached) {
            let Some(samples) = samples else { continue };
            while frame_index < source_start + region.end {
                if frame_index % 1_024 == 0 {
                    decode::checkpoint(cancel, started)?;
                }
                let frame = format.frame(&mut decoder)?;
                if frame_index >= source_start + region.start {
                    samples.extend_from_slice(&frame[..usize::from(format.channels.get())]);
                }
                frame_index += 1;
            }
        }
        decode::checkpoint(cancel, started)?;
        Ok(Self {
            data: Arc::new(Data {
                path,
                format,
                source_start,
                duration_ms,
                schedule,
                cached,
                cache_bytes,
            }),
        })
    }

    #[must_use]
    pub fn cached_bytes(&self) -> usize {
        self.data.cache_bytes
    }

    #[must_use]
    pub fn duration_ms(&self) -> u64 {
        self.data.duration_ms
    }

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.data.format.rate.get()
    }

    pub(crate) fn boundary_ms(&self, tick: u64) -> u64 {
        let boundary = self
            .data
            .schedule
            .regions()
            .iter()
            .find(|r| r.end > tick)
            .map_or(self.data.schedule.duration(), |r| {
                if tick < r.start { r.start } else { r.end }
            });
        (boundary * 1_000).div_ceil(u64::from(self.sample_rate()))
    }

    /// Start a new local pass; preparation never advances the consumer cursor.
    /// # Errors
    /// Reject an out-of-range position, cancellation, decoder failure or busy worker budget.
    pub fn source(
        &self,
        position_ms: u64,
        cancel: &AtomicBool,
    ) -> Result<(PerformanceSource, PerformanceControl), String> {
        if position_ms > self.data.duration_ms {
            return Err("播放位置超出音乐范围".into());
        }
        PerformanceSource::new(
            self.data.clone(),
            self.data.format.frames(position_ms)?,
            cancel,
        )
    }
}

fn allocate(schedule: &LoopSchedule, format: Format) -> Result<(CachedRegions, usize), String> {
    let sizes = schedule
        .regions()
        .iter()
        .map(|region| {
            if region.plays == LoopPlays::Count(1) {
                return Ok(0);
            }
            usize::try_from(region.end - region.start)
                .ok()
                .and_then(|frames| frames.checked_mul(usize::from(format.channels.get())))
                .ok_or_else(|| "循环缓存长度超出范围".to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let bytes = sizes.iter().try_fold(0usize, |total, size| {
        size.checked_mul(size_of::<f32>())
            .and_then(|bytes| total.checked_add(bytes))
            .filter(|&bytes| bytes <= MAX_PERFORMANCE_CACHE_BYTES)
            .ok_or("全部演出循环的解码缓存不能超过 64 MiB，请缩短重复范围")
    })?;
    let cached = sizes
        .into_iter()
        .map(|size| {
            if size == 0 {
                return Ok(None);
            }
            let mut samples = Vec::new();
            samples
                .try_reserve_exact(size)
                .map_err(|_| "无法分配演出循环缓存")?;
            Ok(Some(samples))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((cached, bytes))
}
