//! Prepared host-side source group. Caller owns scheduling, permissions and physical output.
//! No threads, clocks, transport, storage or UI are created here.
mod commands;
mod composition;
mod prepare;
mod source;
mod types;

use source::Entry;
use stagemaster_engine::live::LiveMixer;
pub use stagemaster_playback::Command;
use stagemaster_project::{LiveOutput, LiveSequencePlayer};
pub use types::{Change, Frame, Key, SourceInfo, SourceSpec};

pub struct Session {
    boot: [u8; 16],
    mixer: LiveMixer,
    output: LiveOutput,
    sources: Vec<Entry>,
    claims: Vec<composition::Claim>,
    now_ms: u64,
    sequence: u64,
    frame: Option<Frame>,
    fault: Option<String>,
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
                status: s.player.as_ref().map(LiveSequencePlayer::status),
                step: s.player.as_ref().and_then(LiveSequencePlayer::index),
            })
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
