//! Explicit development policy; all current images retain the disabled physical output.
use stagemaster_runtime::{Denial, Permission, PlaybackPolicy};

pub(super) struct Playback;
impl PlaybackPolicy for Playback {
    fn authorize(&mut self, _permission: Permission) -> Result<(), Denial> {
        if cfg!(feature = "runtime-gatt") {
            Ok(())
        } else {
            Err(Denial::Restricted)
        }
    }
}
pub(super) fn now() -> u64 {
    embassy_time::Instant::now().as_millis()
}
