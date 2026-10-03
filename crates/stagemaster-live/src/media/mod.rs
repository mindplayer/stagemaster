//! Media clocks feed authored positions; host scheduling/leases always keep their own time.
mod admission;
mod catch_up;
mod commands;
mod preparation;
mod prepare;
mod progress;
mod types;

use crate::Session;
pub use preparation::Preparer;
pub use progress::MediaProgress;
use stagemaster_time::Clock;
pub use types::{GroupInfo, GroupKey, GroupSpec, Limits, Prepared, Sample, Status};

pub(crate) struct Group {
    spec: GroupSpec,
    members: Vec<usize>,
    generation: u64,
    status: Status,
    last: Option<Sample>,
    observed_host_ns: u64,
    looping: bool,
}
impl Group {
    pub(crate) fn player_status(
        &self,
        original: stagemaster_playback::Status,
    ) -> stagemaster_playback::Status {
        if self.status == Status::Lost && original == stagemaster_playback::Status::Running {
            stagemaster_playback::Status::Paused
        } else {
            original
        }
    }
}
impl Session {
    /// The adapter must use this same origin when constructing host-side measurements.
    #[must_use]
    pub const fn host_clock(&self) -> Clock {
        self.clock
    }
    #[must_use]
    pub fn media_key(&self, id: [u8; 16]) -> Option<GroupKey> {
        let index = self.media.iter().position(|g| g.spec.id == id)?;
        Some(GroupKey {
            boot: self.boot,
            index,
            generation: self.media[index].generation,
        })
    }
    pub fn media_groups(&self) -> impl Iterator<Item = GroupInfo> + '_ {
        self.media
            .iter()
            .enumerate()
            .map(|(index, group)| GroupInfo {
                key: GroupKey {
                    boot: self.boot,
                    index,
                    generation: group.generation,
                },
                id: group.spec.id,
                provider: group.spec.clock,
                status: group.status,
                position_ms: group.last.map_or(0, |s| s.position_ms),
                sequence: group.last.map_or(0, |s| s.sequence),
            })
    }
    fn media_index(&self, key: GroupKey) -> Result<usize, String> {
        if key.boot != self.boot
            || self
                .media
                .get(key.index)
                .is_none_or(|g| g.generation != key.generation)
        {
            return Err("同步组执行实例或播放代次已失效".into());
        }
        Ok(key.index)
    }
}
pub(crate) fn nanos(millis: u64) -> Result<u64, String> {
    millis
        .checked_mul(1_000_000)
        .ok_or_else(|| "宿主时间超出纳秒范围".into())
}
