use stagemaster_playback::LoopPosition;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Single audio writer; bounded snapshot retries on control threads. No callback locks.
pub(super) struct PublishedPosition {
    sequence: AtomicU64,
    tick: AtomicU64,
    region: AtomicU64,
    pass: AtomicU64,
    exit: AtomicBool,
    ended: AtomicBool,
}

impl PublishedPosition {
    pub fn new(position: LoopPosition) -> Self {
        let result = Self {
            sequence: AtomicU64::new(0),
            tick: AtomicU64::new(0),
            region: AtomicU64::new(u64::MAX),
            pass: AtomicU64::new(0),
            exit: AtomicBool::new(false),
            ended: AtomicBool::new(false),
        };
        result.publish(position);
        result
    }

    pub fn publish(&self, position: LoopPosition) {
        // SeqCst keeps every payload access inside this writer's odd/even bracket.
        self.sequence.fetch_add(1, Ordering::SeqCst);
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
        for _ in 0..16 {
            let before = self.sequence.load(Ordering::SeqCst);
            if !before.is_multiple_of(2) {
                continue;
            }
            let region = self.region.load(Ordering::SeqCst);
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
                return Ok(value);
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
        let position = Arc::new(PublishedPosition::new(value(0)));
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
}
