//! Prepared host-side source group. Caller owns scheduling, permissions and physical output.
//! No threads, clocks, transport, storage or UI are created here.
mod commands;
mod composition;
pub mod media;
mod player;
mod prepare;
mod source;
mod types;

use source::Entry;
use stagemaster_engine::live::LiveMixer;
pub use stagemaster_playback::Command;
pub use stagemaster_playback::{Phase, Progress};
use stagemaster_project::LiveOutput;
pub use types::{Change, Frame, Key, PlaybackSelection, SourceInfo, SourceSpec};

pub struct Session {
    boot: [u8; 16],
    clock: stagemaster_time::Clock,
    mixer: LiveMixer,
    output: LiveOutput,
    sources: Vec<Entry>,
    claims: Vec<composition::Claim>,
    now_ms: u64,
    sequence: u64,
    frame: Option<Frame>,
    fault: Option<String>,
    media: Vec<media::Group>,
}
impl Session {
    #[must_use]
    pub const fn boot(&self) -> [u8; 16] {
        self.boot
    }
    #[must_use]
    pub const fn observed_ms(&self) -> u64 {
        self.now_ms
    }
    #[must_use]
    pub fn layout_id(&self) -> [u8; 32] {
        self.mixer.layout().id()
    }
    /// Only a newly prepared group can be handed to a new scheduling owner.
    #[must_use]
    pub const fn is_pristine(&self) -> bool {
        self.sequence == 0 && self.fault.is_none()
    }

    /// Last complete software frame. Timestamp is never refreshed by reads or input rejection.
    #[must_use]
    pub const fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }
    #[must_use]
    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }
    #[must_use]
    pub fn key(&self, id: [u8; 16]) -> Option<Key> {
        self.sources
            .iter()
            .position(|s| s.id == id)
            .map(|index| Key {
                boot: self.boot,
                index,
            })
    }
    pub fn sources(&self) -> impl Iterator<Item = SourceInfo> + '_ {
        self.sources
            .iter()
            .enumerate()
            .map(|(index, s)| SourceInfo {
                key: Key {
                    boot: self.boot,
                    index,
                },
                id: s.id,
                level: s.level,
                manual_held: s.player.is_none().then(|| {
                    let mut held = [0; 8];
                    for (index, value) in s.values.iter().enumerate() {
                        if value.is_some() {
                            held[index / 64] |= 1 << (index % 64);
                        }
                    }
                    held
                }),
                status: s.player.as_ref().map(|p| {
                    s.media_group
                        .map_or_else(|| p.status(), |g| self.media[g].player_status(p.status()))
                }),
                step: s.player.as_ref().and_then(player::Player::index),
                progress: s
                    .media_group
                    .is_none()
                    .then(|| s.player.as_ref().and_then(player::Player::progress))
                    .flatten(),
            })
    }
    /// Borrow the manual contribution before source level and mixing. No clock advance or allocation.
    /// Playback sources and foreign keys have no manual contribution.
    #[must_use]
    pub fn manual_values(&self, key: Key) -> Option<&[Option<u16>]> {
        if key.boot != self.boot {
            return None;
        }
        let source = self.sources.get(key.index)?;
        source.player.is_none().then_some(source.values.as_slice())
    }
    /// Semantic values and ownership from the last complete software composition.
    #[must_use]
    pub fn values(&self) -> Option<&[u16]> {
        self.frame.as_ref().map(|_| self.output.values())
    }
    #[must_use]
    pub fn winner(&self, attribute: usize) -> Option<[u8; 16]> {
        self.frame.as_ref()?;
        let handle = self.output.winners().get(attribute).copied().flatten()?;
        self.sources
            .iter()
            .find(|s| s.handle == handle)
            .map(|s| s.id)
    }
    fn ready(&self, now_ms: u64) -> Result<(), String> {
        if self.fault.is_some() {
            return Err("来源组已故障，须重新准备".into());
        }
        if now_ms < self.now_ms {
            return Err("播放时钟不能倒退".into());
        }
        if !self.media.is_empty() {
            media::nanos(now_ms)?;
        }
        Ok(())
    }
    fn index(&self, key: Key) -> Result<usize, String> {
        if key.boot != self.boot || key.index >= self.sources.len() {
            return Err("来源不属于当前执行实例".into());
        }
        Ok(key.index)
    }
    fn finish(&mut self, result: Result<(), String>) -> Result<(), String> {
        if let Err(error) = &result {
            self.frame = None;
            self.fault = Some(error.clone());
        }
        result
    }
}
#[cfg(test)]
mod tests;
