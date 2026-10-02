use crate::{
    LoopExitIntent, PerformanceAudio, PerformanceControl, PerformanceSnapshot, PerformanceSource,
};
use serde::Serialize;
use stagemaster_playback::LoopPosition;
use std::{
    cell::Cell,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformancePosition {
    pub instance: Option<String>,
    pub region: Option<usize>,
    pub pass: Option<String>,
    pub exit_requested: bool,
    pub pending_exit: Option<PendingExit>,
    pub control_problem: Option<String>,
    pub ended: bool,
    pub snapshot_pending: bool,
    pub boundary_ms: u64,
    pub cached_bytes: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingExit {
    pub region: usize,
    pub pass: String,
    pub requested: bool,
}

impl From<LoopExitIntent> for PendingExit {
    fn from(value: LoopExitIntent) -> Self {
        Self {
            region: value.region,
            pass: value.pass.to_string(),
            requested: value.requested,
        }
    }
}

pub(super) struct Voice {
    pub source: Option<PerformanceSource>,
    pub control: PerformanceControl,
    pub instance: String,
    last: Cell<PerformanceSnapshot>,
}

impl Voice {
    pub fn new(source: PerformanceSource, control: PerformanceControl) -> Result<Self, String> {
        let snapshot = control.snapshot()?;
        let instance = NEXT_INSTANCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .map_err(|_| "播放实例编号已耗尽，请重新启动应用")?
            .to_string();
        Ok(Self {
            source: Some(source),
            control,
            instance,
            last: Cell::new(snapshot),
        })
    }

    pub fn snapshot(&self) -> (PerformanceSnapshot, bool) {
        if let Ok(value) = self.control.snapshot() {
            self.last.set(value);
            (value, false)
        } else {
            (self.last.get(), true)
        }
    }
}

impl Drop for Voice {
    fn drop(&mut self) {
        self.control.cancel();
    }
}

pub(super) struct Performance {
    pub audio: PerformanceAudio,
    pub initial: LoopPosition,
    pub voice: Option<Voice>,
}

impl Performance {
    pub fn new(audio: PerformanceAudio, voice: Voice) -> Self {
        let initial = voice.snapshot().0.position;
        Self {
            audio,
            initial,
            voice: Some(voice),
        }
    }

    pub fn observe(&self) -> (u64, PerformancePosition, Option<String>) {
        let (snapshot, pending) = self.voice.as_ref().map_or(
            (
                PerformanceSnapshot {
                    position: self.initial,
                    pending_exit: None,
                    control_problem: None,
                    problem: None,
                    stopped: true,
                },
                false,
            ),
            Voice::snapshot,
        );
        let position = snapshot.position;
        let time = if position.ended {
            self.audio.duration_ms()
        } else {
            position.tick * 1_000 / u64::from(self.audio.sample_rate())
        };
        (
            time,
            PerformancePosition {
                instance: self.voice.as_ref().map(|v| v.instance.clone()),
                region: position.region,
                pass: position.pass.map(|p| p.to_string()),
                exit_requested: position.exit_requested,
                pending_exit: snapshot.pending_exit.map(Into::into),
                control_problem: snapshot.control_problem.map(str::to_owned),
                ended: position.ended,
                snapshot_pending: pending,
                boundary_ms: self.audio.boundary_ms(position.tick),
                cached_bytes: self.audio.cached_bytes(),
            },
            snapshot.problem.map(str::to_owned),
        )
    }
}
