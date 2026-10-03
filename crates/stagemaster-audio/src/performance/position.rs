use crate::Consumption;
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
    sequence: AtomicU64,
    tick: AtomicU64,
    region: AtomicU64,
    pass: AtomicU64,
    exit: AtomicBool,
    ended: AtomicBool,
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
            sequence: AtomicU64::new(0),
            tick: AtomicU64::new(0),
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

    pub fn consume(&self, position: LoopPosition) -> Result<(), &'static str> {
        let frames = self
            .frames
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or("音频消费计数已耗尽")?;
        let sampled_ns =
            u64::try_from(self.origin.elapsed().as_nanos()).map_err(|_| "音频采样时间超出范围")?;
        self.write(position, frames, sampled_ns);
        Ok(())
    }

    pub fn publish(&self, position: LoopPosition) {
        self.write(
            position,
            self.frames.load(Ordering::Relaxed),
            self.sampled_ns.load(Ordering::Relaxed),
        );
    }

    fn write(&self, position: LoopPosition, frames: u64, sampled_ns: u64) {
        // SeqCst keeps every payload access inside this writer's odd/even bracket.
        self.sequence.fetch_add(1, Ordering::SeqCst);
        self.frames.store(frames, Ordering::SeqCst);
        self.sampled_ns.store(sampled_ns, Ordering::SeqCst);
        self.tick.store(position.tick, Ordering::SeqCst);
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
        self.observe().map(|value| value.0)
    }

    pub fn observe(&self) -> Result<(LoopPosition, Option<Consumption>), String> {
        for _ in 0..16 {
            let before = self.sequence.load(Ordering::SeqCst);
            if !before.is_multiple_of(2) {
                continue;
            }
            let region = self.region.load(Ordering::SeqCst);
            let frames = self.frames.load(Ordering::SeqCst);
            let sampled_ns = self.sampled_ns.load(Ordering::SeqCst);
            let value = LoopPosition {
                tick: self.tick.load(Ordering::SeqCst),
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
                let consumption = if frames == 0 {
                    None
                } else {
                    Some(Consumption {
                        instance: self.instance,
                        frames,
                        sample_rate: self.sample_rate,
                        at: self
                            .origin
                            .checked_add(Duration::from_nanos(sampled_ns))
                            .ok_or("音频采样时间超出范围")?,
                    })
                };
                return Ok((value, consumption));
            }
        }
        Err("音频游标正在更新，请重试读取".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn value(tick: u64) -> LoopPosition {
        LoopPosition {
            tick,
            region: Some(usize::try_from(tick % 128).unwrap()),
            pass: Some(tick + 1),
            exit_requested: tick.is_multiple_of(2),
            ended: false,
        }
    }

    #[test]
    fn concurrent_readers_never_observe_a_torn_position_and_final_state_is_available() {
        let position = Arc::new(PublishedPosition::new(value(0), 48_000).unwrap());
        let writer = position.clone();
        let thread = std::thread::spawn(move || {
            for tick in 1..=100_000 {
                writer.publish(value(tick));
            }
        });
        for _ in 0..100_000 {
            if let Ok(read) = position.read() {
                assert_eq!(read, value(read.tick));
            }
        }
        thread.join().unwrap();
        assert_eq!(position.read().unwrap(), value(100_000));
    }

    #[test]
    fn concurrent_consumption_keeps_time_frame_count_and_loop_position_together() {
        let position = Arc::new(PublishedPosition::new(value(0), 48_000).unwrap());
        let writer = position.clone();
        let started = Instant::now();
        let thread = std::thread::spawn(move || {
            for frame in 1..=100_000 {
                writer.consume(value(frame)).unwrap();
            }
        });
        let mut previous = None::<Consumption>;
        for _ in 0..100_000 {
            if let Ok((read, Some(stamp))) = position.observe() {
                assert_eq!(read, value(stamp.frames));
                assert_eq!(stamp.instance, position.instance());
                assert_eq!(stamp.sample_rate, 48_000);
                assert!(stamp.at >= started && stamp.at <= Instant::now());
                if let Some(old) = previous {
                    assert!(stamp.frames >= old.frames && stamp.at >= old.at);
                }
                previous = Some(stamp);
            }
        }
        thread.join().unwrap();
        let (read, stamp) = position.observe().unwrap();
        assert_eq!(read, value(100_000));
        assert_eq!(stamp.unwrap().frames, 100_000);
    }

    #[test]
    fn initial_busy_metadata_and_exhausted_consumption_cannot_invent_fresh_frames() {
        let position = PublishedPosition::new(value(0), 48_000).unwrap();
        assert!(PublishedPosition::new(value(0), 0).is_err());
        assert!(position.observe().unwrap().1.is_none());
        position.consume(value(1)).unwrap();
        let before = position.observe().unwrap().1;
        let mut metadata = value(1);
        metadata.exit_requested = !metadata.exit_requested;
        position.publish(metadata);
        assert_eq!(position.observe().unwrap(), (metadata, before));
        position.frames.store(u64::MAX, Ordering::Relaxed);
        let exhausted = position.observe().unwrap();
        assert!(position.consume(value(2)).is_err());
        assert_eq!(position.observe().unwrap(), exhausted);
        position.block_for_test();
        assert!(position.observe().is_err());
    }
}
