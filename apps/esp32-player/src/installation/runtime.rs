//! Actual installation firmware never receives permission to start physical playback.
use stagemaster_runtime::{Denial, Permission, PlaybackPolicy};

pub(super) struct DisabledPlayback;
impl PlaybackPolicy for DisabledPlayback {
    fn authorize(&mut self, _permission: Permission) -> Result<(), Denial> {
        Err(Denial::Restricted)
    }
}
pub(super) fn now() -> u64 {
    embassy_time::Instant::now().as_millis()
}
