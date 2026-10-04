use stagemaster_playback::Status;
use stagemaster_project::PackageSelection;

#[derive(Clone, Debug)]
pub struct SourceSpec {
    pub id: [u8; 16],
    pub priority: i16,
    /// None registers a trusted manual attribute layer.
    pub playback: Option<PlaybackSelection>,
}
/// Process-local source identity. Caller must supply a fresh boot ID at every preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub(crate) boot: [u8; 16],
    pub(crate) index: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub attribute: usize,
    /// None releases ownership; Some(0) still owns zero. Values are already translated semantics.
    pub value: Option<u16>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub sequence: u64,
    pub sampled_ms: u64,
    pub universe: u16,
    pub slots: [u8; 512],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceInfo {
    pub key: Key,
    pub id: [u8; 16],
    pub level: u16,
    pub status: Option<Status>,
    pub step: Option<usize>,
    pub progress: Option<stagemaster_playback::Progress>,
    /// Manual contribution ownership, independent of fader level and final mix winners.
    pub manual_held: Option<[u64; 8]>,
}

/// Host preparation choices; device package selections and wire formats remain separate.
#[derive(Clone, Debug)]
pub enum PlaybackSelection {
    Program(PackageSelection),
    AudioTimeline,
}
impl From<PackageSelection> for PlaybackSelection {
    fn from(value: PackageSelection) -> Self {
        Self::Program(value)
    }
}
impl PlaybackSelection {
    #[must_use]
    pub const fn program(&self) -> Option<&PackageSelection> {
        match self {
            Self::Program(p) => Some(p),
            Self::AudioTimeline => None,
        }
    }
}
