use super::{ControlSpec, Lane, Mailbox, MediaCommand};
use crate::{LiveBackend, media::MediaPort};
use stagemaster_live::{Session, media::GroupKey};
use stagemaster_runtime::Code;
use std::sync::{Arc, Mutex};

impl LiveBackend {
    /// Register bounded provider control before transferring this backend to the original Host.
    /// # Errors
    /// Reject unknown/duplicate groups, invalid bounds or non-pristine execution.
    pub fn with_controlled_media(
        session: Session,
        specs: &[ControlSpec],
    ) -> Result<(Self, Vec<MediaPort>), Code> {
        let (mut backend, mut ports) = Self::with_media(session)?;
        for spec in specs {
            if spec.duration_ms == 0
                || spec.duration_ms > stagemaster_playback::MAX_TIME_MS
                || !(1..=60_000).contains(&spec.timeout_ms)
            {
                return Err(Code::State);
            }
            let index = ports
                .iter()
                .position(|p| p.initial().id == spec.group)
                .ok_or(Code::Selection)?;
            if backend.media[index].control.is_some() {
                return Err(Code::State);
            }
            let slot = Arc::new(Mutex::new(Mailbox::default()));
            backend.media[index].control = Some(Lane {
                spec: *spec,
                slot: slot.clone(),
                state: None,
            });
            ports[index].control = Some(slot);
        }
        Ok((backend, ports))
    }

    pub(crate) fn request_media(
        &mut self,
        key: GroupKey,
        command: MediaCommand,
        now: u64,
    ) -> Result<(), Code> {
        let index = self
            .session
            .media_groups()
            .position(|g| g.key == key)
            .ok_or(Code::Selection)?;
        self.media[index]
            .control
            .as_mut()
            .ok_or(Code::State)?
            .admit(key, command, now)
    }

    pub(crate) fn direct_media_allowed(&self, key: GroupKey) -> Result<(), Code> {
        let worker = self
            .media
            .iter()
            .find(|w| w.key.same_group(key))
            .ok_or(Code::Selection)?;
        if worker.control.is_some() {
            Err(Code::State)
        } else {
            Ok(())
        }
    }
}
