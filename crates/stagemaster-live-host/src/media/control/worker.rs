use super::{ControlFailure, Lane, MediaCommand};
use crate::media::slots::Worker;
use stagemaster_live::{Session, media::Status};
use stagemaster_runtime::Code;

impl Worker {
    pub(crate) fn poll_requested_activation(
        &mut self,
        session: &mut Session,
        now: u64,
    ) -> Result<(), Code> {
        let Some(lane) = &mut self.control else {
            return Ok(());
        };
        let Ok(mut staging) = self.staging.try_lock() else {
            return Ok(());
        };
        let Some(entry) = staging.entry.as_mut().filter(|e| e.result.is_none()) else {
            return Ok(());
        };
        let Some(ticket) = entry.request else {
            return Ok(());
        };
        let Ok(mut slot) = lane.slot.try_lock() else {
            return Ok(());
        };
        match slot.pending(ticket) {
            Ok(request) if now < request.deadline_ms && !slot.activated => {
                let result = entry.activate(session, now);
                if result.is_ok() {
                    slot.activated = true;
                } else if let Some(state) = &mut slot.state {
                    state.result = Some(result.map_err(ControlFailure::Provider));
                }
            }
            _ => {
                entry.result = Some(Err(Code::State));
            }
        }
        lane.state = slot.state;
        if session.fault().is_some() {
            Err(Code::Playback)
        } else {
            Ok(())
        }
    }
}

impl Lane {
    pub fn poll_completion(&mut self, session: &mut Session, now: u64) -> Result<(), Code> {
        let Ok(mut slot) = self.slot.try_lock() else {
            return Ok(());
        };
        if let Some(mut state) = slot.state.filter(|s| s.result.is_none()) {
            if now >= state.request.deadline_ms {
                state.result = Some(Err(ControlFailure::TimedOut));
                slot.completion = None;
            } else if let Some((ticket, result)) = slot.completion.take()
                && ticket == state.request.ticket
            {
                let result = result.and_then(|()| {
                    complete(
                        session,
                        self.spec.group,
                        state.request.command,
                        slot.activated,
                        now,
                    )
                });
                state.result = Some(result.map_err(ControlFailure::Provider));
            }
            slot.state = Some(state);
        }
        self.state = slot.state;
        if session.fault().is_some() {
            Err(Code::Playback)
        } else {
            Ok(())
        }
    }
}

fn complete(
    session: &mut Session,
    id: [u8; 16],
    command: MediaCommand,
    activated: bool,
    now: u64,
) -> Result<(), Code> {
    let group = session
        .media_groups()
        .find(|g| g.id == id)
        .ok_or(Code::Selection)?;
    let playing = match command {
        MediaCommand::Stop => return session.stop_media(group.key, now).map_err(|_| Code::State),
        MediaCommand::Play => true,
        MediaCommand::Pause => false,
        MediaCommand::Seek {
            position_ms,
            playing,
        } => {
            if !activated || group.position_ms < position_ms {
                return Err(Code::State);
            }
            playing
        }
    };
    let expected = if playing {
        Status::Following
    } else {
        Status::Paused
    };
    if group.status == expected {
        Ok(())
    } else {
        Err(Code::State)
    }
}
