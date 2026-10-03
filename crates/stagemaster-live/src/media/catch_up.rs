use super::Prepared;
use crate::Command;
use stagemaster_playback::MAX_TIME_MS;

impl Prepared {
    /// Catch a newly compiled plan up to a later actual sample before queue admission.
    /// Must run off the scheduler. This consumes failed plans there rather than admitting a partial group.
    /// The caller pairs the returned position/state with the SAME real provider observation.
    /// # Errors
    /// Reject backwards seeking, out-of-range position or playback failure.
    pub fn advance_to(mut self, position_ms: u64, playing: bool) -> Result<Self, String> {
        if position_ms < self.position_ms || position_ms > MAX_TIME_MS {
            return Err("准备后的回退定位须重新编译同步组".into());
        }
        for (_, player) in &mut self.players {
            if !self.playing {
                player.apply_timeline(Command::Resume, self.position_ms)?;
            }
            player.apply_timeline(Command::Advance, position_ms)?;
            if !playing {
                player.apply_timeline(Command::Pause, position_ms)?;
            }
        }
        self.position_ms = position_ms;
        self.playing = playing;
        Ok(self)
    }
}
