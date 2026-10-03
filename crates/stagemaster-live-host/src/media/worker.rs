use super::{
    MediaState,
    slots::{Activation, Entry, ObservationReceipt, Worker},
};
use crate::LiveBackend;
use stagemaster_live::Session;
use stagemaster_runtime::Code;

impl Worker {
    pub(crate) fn activate(
        &self,
        ticket: Activation,
        session: &mut Session,
        now: u64,
    ) -> Result<(), Code> {
        let mut slot = self.staging.try_lock().map_err(|_| Code::Busy)?;
        let entry = slot
            .entry
            .as_mut()
            .filter(|e| e.ticket == ticket && e.result.is_none())
            .ok_or(Code::State)?;
        entry.activate(session, now)
    }
    pub(crate) fn poll(&mut self, session: &mut Session, now: u64) -> Result<(), Code> {
        self.poll_requested_activation(session, now)?;
        let update = match self.inbox.try_lock() {
            Ok(mut inbox) => inbox.latest.take(),
            Err(_) => None,
        };
        if let Some(update) = update {
            let result = session
                .observe_media(update.key, update.sample, &update.mapping, now)
                .map_err(|_| {
                    if session.fault().is_some() {
                        Code::Playback
                    } else {
                        Code::State
                    }
                });
            self.receipt = Some(ObservationReceipt {
                serial: update.serial,
                key: update.key,
                sample_sequence: update.sample.sequence,
                result,
            });
            if session.fault().is_some() {
                return Err(Code::Playback);
            }
        }
        if let Some(control) = &mut self.control {
            control.poll_completion(session, now)?;
        }
        Ok(())
    }
}
impl LiveBackend {
    pub(crate) fn activate_media(&mut self, ticket: Activation, now: u64) -> Result<(), Code> {
        self.direct_media_allowed(ticket.group)?;
        let worker = self
            .media
            .iter()
            .find(|w| w.key.same_group(ticket.group))
            .ok_or(Code::Selection)?;
        worker.activate(ticket, &mut self.session, now)
    }
    pub(crate) fn media_state(&self) -> [Option<MediaState>; 64] {
        let mut states = [None; 64];
        for (index, group) in self.session.media_groups().enumerate() {
            states[index] = Some(MediaState {
                group,
                observation: self.media.get(index).and_then(|w| w.receipt),
                control: self
                    .media
                    .get(index)
                    .and_then(|w| w.control.as_ref())
                    .and_then(|c| c.state),
            });
        }
        states
    }
}

impl Entry {
    pub(super) fn activate(&mut self, session: &mut Session, now: u64) -> Result<(), Code> {
        let result = session
            .activate_media(&mut self.prepared, self.sample, &self.mapping, now)
            .map_err(|_| {
                if session.fault().is_some() {
                    Code::Playback
                } else {
                    Code::State
                }
            });
        self.result = Some(result);
        result
    }
}
