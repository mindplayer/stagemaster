//! Host-only composite lighting. This type cannot be exported as a device single Plan.
use crate::CompiledOutput;
use stagemaster_playback::{CrossfadeTiming, Plan, Player, SceneCrossfade};

pub struct CompiledAudioSegment {
    pub playback: AudioSegmentPlan,
    pub output: CompiledOutput,
}
pub enum AudioSegmentPlan {
    Single(Plan),
    Crossfade {
        source: Plan,
        target: Plan,
        timing: CrossfadeTiming,
    },
}
pub enum AudioSegmentPlayer {
    Single(Player),
    Crossfade(SceneCrossfade),
}
impl AudioSegmentPlan {
    /// # Errors
    /// Rejects incompatible source bindings or invalid transition clocks.
    pub fn into_player(self) -> Result<AudioSegmentPlayer, String> {
        Ok(match self {
            Self::Single(plan) => {
                let mut player = Player::new(plan, 0);
                player.execute(0, 0)?;
                AudioSegmentPlayer::Single(player)
            }
            Self::Crossfade {
                source,
                target,
                timing,
            } => AudioSegmentPlayer::Crossfade(SceneCrossfade::new(source, target, timing)?),
        })
    }
}
impl AudioSegmentPlayer {
    /// The host supplies elapsed time from its sole audio cursor; reconstruct after a backwards seek.
    /// # Errors
    /// Rejects backwards single-plan time and out-of-range crossfade time.
    pub fn advance(&mut self, elapsed: u64) -> Result<(), String> {
        match self {
            Self::Single(player) => player.advance(elapsed),
            Self::Crossfade(player) => player.sample(elapsed).map(|_| ()),
        }
    }
    #[must_use]
    pub fn values(&self) -> &[u16] {
        match self {
            Self::Single(player) => player.values(),
            Self::Crossfade(player) => player.values(),
        }
    }
}
