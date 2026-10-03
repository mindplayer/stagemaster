use stagemaster_project::LiveSequencePlayer;
use stagemaster_time::{Clock, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_age_ns: u64,
    pub max_uncertainty_ns: u64,
    pub max_gap_ms: u64,
    pub max_rate_percent: u16,
    /// Explicit cursor resolution tolerance; not a measured hardware synchronization accuracy.
    pub position_tolerance_ms: u64,
}
#[derive(Clone, Debug)]
pub struct GroupSpec {
    pub id: [u8; 16],
    pub clock: Clock,
    pub sources: Vec<[u8; 16]>,
    pub limits: Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupKey {
    pub(super) boot: [u8; 16],
    pub(super) index: usize,
    pub(super) generation: u64,
}
impl GroupKey {
    /// Compare fixed membership, deliberately ignoring the current playback generation.
    #[must_use]
    pub fn same_group(self, other: Self) -> bool {
        self.boot == other.boot && self.index == other.index
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ready,
    Following,
    Paused,
    Lost,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub at: Instant,
    pub sequence: u64,
    pub position_ms: u64,
    pub playing: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupInfo {
    pub key: GroupKey,
    pub id: [u8; 16],
    pub provider: Clock,
    pub status: Status,
    pub position_ms: u64,
    pub sequence: u64,
}
/// Prepared off the scheduling path. On activation this retains replaced players.
/// Dispose it off the scheduler after success OR rejection. Fields cannot be forged.
pub struct Prepared {
    pub(super) key: GroupKey,
    pub(super) deadline_ms: u64,
    pub(super) position_ms: u64,
    pub(super) playing: bool,
    pub(super) players: Vec<(usize, LiveSequencePlayer)>,
}
impl Prepared {
    #[must_use]
    pub const fn key(&self) -> GroupKey {
        self.key
    }
}
