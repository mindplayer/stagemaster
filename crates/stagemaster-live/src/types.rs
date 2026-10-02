use stagemaster_playback::Status;
use stagemaster_project::PackageSelection;

#[derive(Clone, Debug)]
pub struct SourceSpec {
    pub id: [u8; 16],
    pub priority: i16,
    /// None registers a trusted manual attribute layer.
    pub playback: Option<PackageSelection>,
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
}
