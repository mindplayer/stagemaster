use super::{
    control::{Failure, PerformanceControl, Shared},
    decode,
    prepare::Data,
    stream::Feed,
};
use rodio::{ChannelCount, SampleRate, Source};
use stagemaster_playback::{LoopPlayback, LoopSchedule};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Audio frames and exported source position share the same deterministic loop cursor.
pub struct PerformanceSource {
    data: Arc<Data>,
    playback: LoopPlayback,
    feed: Feed,
    shared: Arc<Shared>,
    frame: [f32; 2],
    channel: usize,
    failed: bool,
}

impl PerformanceSource {
    pub(super) fn new(
        data: Arc<Data>,
        tick: u64,
        cancel: &AtomicBool,
    ) -> Result<(Self, PerformanceControl), String> {
        let started = Instant::now();
        decode::checkpoint(cancel, started)?;
        let schedule =
            LoopSchedule::new(data.schedule.duration(), data.schedule.regions().to_vec())?;
        let playback = LoopPlayback::new(schedule, tick)?;
        let shared = Shared::new(playback.position());
        let feed = Feed::prepare(data.clone(), tick, shared.cancelled.clone(), cancel)?;
        decode::checkpoint(cancel, started)?;
        let control = PerformanceControl {
            shared: shared.clone(),
        };
        let source = Self {
            data,
            playback,
            feed,
            shared,
            frame: [0.0; 2],
            channel: 0,
            failed: false,
        };
        source.check_ready()?;
        Ok((source, control))
    }

    /// Recheck immediately before a host replaces its previous usable source.
    /// # Errors
    /// Reject already-known background decode failure, cancellation or consumer failure.
    pub fn check_ready(&self) -> Result<(), String> {
        if self.failed || self.feed.has_failed() {
            return Err("音乐解码已失败，原播放状态保持".into());
        }
        if self.shared.cancelled.load(Ordering::Acquire) {
            return Err("音乐准备已取消".into());
        }
        Ok(())
    }

    fn next_frame(&mut self) -> Result<[f32; 2], Failure> {
        if self.shared.cancelled.load(Ordering::Acquire) {
            return Err(Failure::Cancelled);
        }
        self.shared.apply(&mut self.playback)?;
        let position = self.playback.position();
        let channels = usize::from(self.data.format.channels.get());
        if let Some(index) = position.region
            && let Some(cache) = &self.data.cached[index]
        {
            let offset = usize::try_from(position.tick - self.data.schedule.regions()[index].start)
                .map_err(|_| Failure::Sequence)?
                * channels;
            let mut frame = [0.0; 2];
            frame[..channels].copy_from_slice(
                cache
                    .get(offset..offset + channels)
                    .ok_or(Failure::Sequence)?,
            );
            return Ok(frame);
        }
        self.feed.frame(position.tick, channels)
    }

    fn fail(&mut self, failure: Failure) -> Option<f32> {
        self.failed = true;
        self.shared.fail(failure);
        self.feed.cancel();
        None
    }
}

impl Iterator for PerformanceSource {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.playback.position().ended {
            return None;
        }
        if self.channel == 0 {
            match self.next_frame() {
                Ok(frame) => self.frame = frame,
                Err(failure) => return self.fail(failure),
            }
        }
        let sample = self.frame[self.channel];
        self.channel += 1;
        if self.channel == usize::from(self.data.format.channels.get()) {
            self.channel = 0;
            match self.playback.advance(1) {
                Ok(position) => self.shared.publish(position),
                Err(_) => {
                    self.fail(Failure::Cursor);
                }
            }
        }
        Some(sample)
    }
}

impl Source for PerformanceSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        self.data.format.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.data.format.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Drop for PerformanceSource {
    fn drop(&mut self) {
        self.shared.cancelled.store(true, Ordering::Release);
        self.shared.stopped.store(true, Ordering::Release);
    }
}
