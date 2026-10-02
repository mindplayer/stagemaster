//! Persisted transition origins. Scene identities and semantic values, never DMX slots.
use crate::ClipEntryFade;
use serde::{Deserialize, Serialize};

pub(super) const CAPABILITY: &str = "media.audio-clip-crossfade";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipFadeMode {
    #[default]
    Snapshot,
    Dynamic,
}
impl ClipFadeMode {
    #[allow(clippy::trivially_copy_pass_by_ref)] // Serde requires borrowed fields.
    pub(super) fn is_snapshot(&self) -> bool {
        *self == Self::Snapshot
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipCrossfadeSource {
    pub scene_id: Option<String>,
    pub effect_offset_ms: u64,
    pub elapsed_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_fade: Option<ClipEntryFade>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipEntryCrossfade {
    pub duration_ms: u64,
    pub offset_ms: u64,
    pub source: ClipCrossfadeSource,
}
impl ClipEntryCrossfade {
    #[must_use]
    pub fn visible_ms(&self, length: u64) -> u64 {
        self.duration_ms.saturating_sub(self.offset_ms).min(length)
    }
    pub(super) fn shift(&mut self, delta: i128) -> Result<(), String> {
        self.offset_ms = shifted(self.offset_ms, delta)?;
        self.source.elapsed_ms = shifted(self.source.elapsed_ms, delta)?;
        Ok(())
    }
}
fn shifted(time: u64, delta: i128) -> Result<u64, String> {
    u64::try_from(i128::from(time) + delta)
        .map_err(|_| "裁切开始不能早于原交叉零点或来源零点".into())
}
