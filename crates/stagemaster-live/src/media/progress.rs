use super::Sample;

/// Paired with the actual source cursor by a trusted, fixed media provider.
/// Counters reset only when a freshly prepared playback generation is activated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaProgress {
    pub instance: u64,
    pub sample_rate: u32,
    pub position_ticks: u64,
    pub consumed_ticks: u64,
    pub repeated_ticks: u64,
}
impl MediaProgress {
    pub(super) fn validate(self, position_ms: u64) -> Result<(), String> {
        if self.instance == 0
            || self.sample_rate == 0
            || u128::from(self.position_ticks) * 1000 / u128::from(self.sample_rate)
                != u128::from(position_ms)
            || u128::from(self.consumed_ticks)
                > u128::from(self.position_ticks) + u128::from(self.repeated_ticks)
        {
            return Err("媒体素材位置与实际消费观测不一致".into());
        }
        Ok(())
    }
    fn advance(self, previous: Self) -> Result<u128, String> {
        let consumed = self.consumed_ticks.checked_sub(previous.consumed_ticks);
        let repeated = self.repeated_ticks.checked_sub(previous.repeated_ticks);
        let (Some(consumed), Some(repeated)) = (consumed, repeated) else {
            return Err("媒体消费计数倒退，须重新准备播放代次".into());
        };
        if self.instance != previous.instance
            || self.sample_rate != previous.sample_rate
            || (consumed == 0 && repeated != 0)
            || u128::from(previous.position_ticks) + u128::from(consumed)
                != u128::from(self.position_ticks) + u128::from(repeated)
        {
            return Err("媒体位置跳变缺少连续消费依据".into());
        }
        Ok((u128::from(consumed) * 1_000_000_000).div_ceil(u128::from(self.sample_rate)))
    }
}
impl Sample {
    /// Verify a requested millisecond seek against the actual source's quantized sample frame.
    /// The prepared plan uses the observed position, not an invented exact-millisecond cursor.
    #[must_use]
    pub fn matches_seek(self, requested_ms: u64) -> bool {
        match self.progress {
            Some(p) => {
                p.sample_rate > 0
                    && p.consumed_ticks == 0
                    && p.repeated_ticks == 0
                    && u128::from(requested_ms) * u128::from(p.sample_rate) / 1000
                        == u128::from(p.position_ticks)
            }
            None => self.position_ms == requested_ms,
        }
    }
    pub(super) fn progress_ns(self, previous: Self) -> Result<u128, String> {
        match (self.progress, previous.progress) {
            (Some(next), Some(previous)) => next.advance(previous),
            (None, None) => self
                .position_ms
                .checked_sub(previous.position_ms)
                .map(|ms| u128::from(ms) * 1_000_000)
                .ok_or_else(|| "媒体定位须重新准备播放代次".into()),
            _ => Err("媒体观测形态变化，须重新准备播放代次".into()),
        }
    }
    pub(super) fn repeated_since(self, previous: Self) -> bool {
        self.progress
            .zip(previous.progress)
            .is_some_and(|(next, old)| next.repeated_ticks > old.repeated_ticks)
    }
}
