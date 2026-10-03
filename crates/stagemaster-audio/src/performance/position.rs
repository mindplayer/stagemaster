use crate::{Consumption, PlaybackRequest, RenderObservation};
use stagemaster_playback::LoopPosition;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Single audio writer; bounded snapshot retries on control threads. No callback locks.
pub(super) struct PublishedPosition {
    origin: Instant,
    instance: u64,
    sample_rate: u32,
    frames: AtomicU64,
    sampled_ns: AtomicU64,
    render_sequence: AtomicU64,
    render_ns: AtomicU64,
    applied_revision: AtomicU64,
    applied_playing: AtomicBool,
    sequence: AtomicU64,
    tick: AtomicU64,
    repeated_ticks: AtomicU64,
    region: AtomicU64,
    pass: AtomicU64,
    exit: AtomicBool,
    ended: AtomicBool,
}

#[derive(Clone, Copy)]
struct Stamps {
    frames: u64,
    sampled_ns: u64,
    rendered: u64,
    render_ns: u64,
    applied: PlaybackRequest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Publication {
    pub position: LoopPosition,
    pub consumption: Option<Consumption>,
    pub render: Option<RenderObservation>,
}

impl PublishedPosition {
    #[cfg(test)]
    pub(crate) fn block_for_test(&self) {
        self.sequence.fetch_or(1, Ordering::SeqCst);
    }
    pub fn new(position: LoopPosition, sample_rate: u32) -> Result<Self, String> {
        if sample_rate == 0 {
            return Err("音频采样率不能为零".into());
        }
        let result = Self {
            origin: Instant::now(),
            instance: crate::consumption::next_instance()?,
            sample_rate,
            frames: AtomicU64::new(0),
            sampled_ns: AtomicU64::new(0),
            render_sequence: AtomicU64::new(0),
            render_ns: AtomicU64::new(0),
            applied_revision: AtomicU64::new(0),
            applied_playing: AtomicBool::new(false),
            sequence: AtomicU64::new(0),
            tick: AtomicU64::new(0),
            repeated_ticks: AtomicU64::new(0),
            region: AtomicU64::new(u64::MAX),
            pass: AtomicU64::new(0),
            exit: AtomicBool::new(false),
            ended: AtomicBool::new(false),
        };
        result.publish(position);
        Ok(result)
    }

    pub fn instance(&self) -> u64 {
        self.instance
    }

    pub fn frame(
        &self,
        position: LoopPosition,
        applied: PlaybackRequest,
    ) -> Result<(), &'static str> {
        let mut stamps = self.stamps();
        stamps.rendered = stamps.rendered.checked_add(1).ok_or("音频回调计数已耗尽")?;
        let now_ns =
            u64::try_from(self.origin.elapsed().as_nanos()).map_err(|_| "音频采样时间超出范围")?;
        if applied.playing {
            stamps.frames = stamps.frames.checked_add(1).ok_or("音频消费计数已耗尽")?;
            stamps.sampled_ns = now_ns;
        }
        stamps.render_ns = now_ns;
        stamps.applied = applied;
        self.write(position, stamps);
        Ok(())
    }

    pub fn publish(&self, position: LoopPosition) {
        self.write(position, self.stamps());
    }

    fn stamps(&self) -> Stamps {
        Stamps {
            frames: self.frames.load(Ordering::SeqCst),
            sampled_ns: self.sampled_ns.load(Ordering::SeqCst),
            rendered: self.render_sequence.load(Ordering::SeqCst),
            render_ns: self.render_ns.load(Ordering::SeqCst),
            applied: PlaybackRequest {
                revision: self.applied_revision.load(Ordering::SeqCst),
                playing: self.applied_playing.load(Ordering::SeqCst),
            },
        }
    }

    fn write(&self, position: LoopPosition, stamps: Stamps) {
        // SeqCst keeps every payload access inside this writer's odd/even bracket.
        self.sequence.fetch_add(1, Ordering::SeqCst);
        self.frames.store(stamps.frames, Ordering::SeqCst);
        self.sampled_ns.store(stamps.sampled_ns, Ordering::SeqCst);
        self.render_sequence
            .store(stamps.rendered, Ordering::SeqCst);
        self.render_ns.store(stamps.render_ns, Ordering::SeqCst);
        self.applied_revision
            .store(stamps.applied.revision, Ordering::SeqCst);
        self.applied_playing
            .store(stamps.applied.playing, Ordering::SeqCst);
        self.tick.store(position.tick, Ordering::SeqCst);
        self.repeated_ticks
            .store(position.repeated_ticks, Ordering::SeqCst);
        self.region.store(
            position.region.map_or(u64::MAX, |v| v as u64),
            Ordering::SeqCst,
        );
        self.pass
            .store(position.pass.unwrap_or(0), Ordering::SeqCst);
        self.exit.store(position.exit_requested, Ordering::SeqCst);
        self.ended.store(position.ended, Ordering::SeqCst);
        self.sequence.fetch_add(1, Ordering::SeqCst);
    }

    pub fn read(&self) -> Result<LoopPosition, String> {
        self.observe().map(|value| value.position)
    }

    pub fn observe(&self) -> Result<Publication, String> {
        for _ in 0..16 {
            let before = self.sequence.load(Ordering::SeqCst);
            if !before.is_multiple_of(2) {
                continue;
            }
            let region = self.region.load(Ordering::SeqCst);
            let stamps = self.stamps();
            let value = LoopPosition {
                tick: self.tick.load(Ordering::SeqCst),
                repeated_ticks: self.repeated_ticks.load(Ordering::SeqCst),
                region: (region != u64::MAX)
                    .then(|| usize::try_from(region).ok())
                    .flatten(),
                pass: match self.pass.load(Ordering::SeqCst) {
                    0 => None,
                    pass => Some(pass),
                },
                exit_requested: self.exit.load(Ordering::SeqCst),
                ended: self.ended.load(Ordering::SeqCst),
            };
            if before == self.sequence.load(Ordering::SeqCst) {
                let consumption = if stamps.frames == 0 {
                    None
                } else {
                    Some(Consumption {
                        instance: self.instance,
                        frames: stamps.frames,
                        sample_rate: self.sample_rate,
                        at: self.at(stamps.sampled_ns)?,
                    })
                };
                let render = if stamps.rendered == 0 {
                    None
                } else {
                    Some(RenderObservation {
                        instance: self.instance,
                        sequence: stamps.rendered,
                        sample_rate: self.sample_rate,
                        at: self.at(stamps.render_ns)?,
                        applied: stamps.applied,
                    })
                };
                return Ok(Publication {
                    position: value,
                    consumption,
                    render,
                });
            }
        }
        Err("音频游标正在更新，请重试读取".into())
    }
    fn at(&self, nanos: u64) -> Result<Instant, String> {
        self.origin
            .checked_add(Duration::from_nanos(nanos))
            .ok_or_else(|| "音频采样时间超出范围".into())
    }
}

#[cfg(test)]
mod tests;
