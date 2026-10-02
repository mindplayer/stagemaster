use super::voice::Voice;
use crate::PerformanceAudio;
use stagemaster_playback::LoopSchedule;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) struct Identity {
    pub owner: Arc<()>,
    pub revision: u64,
}

pub struct AudioLoadRequest {
    pub(super) identity: Identity,
    pub(super) file: PathBuf,
    pub(super) in_ms: u64,
    pub(super) duration_ms: u64,
    pub(super) schedule: Option<LoopSchedule>,
}

pub struct AudioLoadTicket {
    pub(super) identity: Identity,
}

impl AudioLoadTicket {
    /// Bind media discovered after a file dialog or verification to the original transport revision.
    /// # Errors
    /// Reject invalid media ranges or a schedule with a different local duration.
    pub fn request(
        self,
        file: PathBuf,
        in_ms: u64,
        out_ms: u64,
        schedule: Option<LoopSchedule>,
    ) -> Result<AudioLoadRequest, String> {
        if in_ms >= out_ms
            || out_ms > crate::MAX_DURATION_MS
            || schedule
                .as_ref()
                .is_some_and(|p| p.duration() != out_ms - in_ms)
        {
            return Err("音乐范围或演出编排长度无效".into());
        }
        Ok(AudioLoadRequest {
            identity: self.identity,
            file,
            in_ms,
            duration_ms: out_ms - in_ms,
            schedule,
        })
    }
}

pub struct PreparedAudioLoad {
    pub(super) identity: Identity,
    pub(super) file: PathBuf,
    pub(super) in_ms: u64,
    pub(super) duration_ms: u64,
    pub(super) performance: Option<(PerformanceAudio, Voice)>,
}

impl AudioLoadRequest {
    /// Prepare an immutable native source without holding a session or opening a sound device.
    /// # Errors
    /// Reject cancellation, invalid media, cache/worker budgets and prefetch failure.
    pub fn prepare(self, cancel: &AtomicBool) -> Result<PreparedAudioLoad, String> {
        if cancel.load(Ordering::Acquire) {
            return Err("音乐准备已取消".into());
        }
        let performance = self
            .schedule
            .map(|plan| {
                let audio =
                    PerformanceAudio::prepare(self.file.clone(), self.in_ms, &plan, cancel)?;
                let (source, control) = audio.source(0, cancel)?;
                Ok::<_, String>((audio, Voice::new(source, control)?))
            })
            .transpose()?;
        Ok(PreparedAudioLoad {
            identity: self.identity,
            file: self.file,
            in_ms: self.in_ms,
            duration_ms: self.duration_ms,
            performance,
        })
    }
}

pub struct AudioSeekRequest {
    pub(super) identity: Identity,
    pub(super) audio: PerformanceAudio,
    pub(super) position_ms: u64,
    pub(super) playing: bool,
}

pub struct PreparedAudioSeek {
    pub(super) identity: Identity,
    pub(super) voice: Voice,
    pub(super) position_ms: u64,
    pub(super) playing: bool,
}

impl AudioSeekRequest {
    /// Reuse the current immutable PCM while a fresh source is prepared outside the session lock.
    /// # Errors
    /// Reject an invalid target, cancelled work, busy workers or decoding/prefetch failure.
    pub fn prepare(self, cancel: &AtomicBool) -> Result<PreparedAudioSeek, String> {
        let (source, control) = self.audio.source(self.position_ms, cancel)?;
        Ok(PreparedAudioSeek {
            identity: self.identity,
            voice: Voice::new(source, control)?,
            position_ms: self.position_ms,
            playing: self.playing,
        })
    }
}
