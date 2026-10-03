use crate::{
    LoopExitIntent, PerformanceAudio, PerformanceControl, PerformanceSnapshot, PerformanceSource,
};
use serde::Serialize;
use stagemaster_playback::LoopPosition;
use std::cell::Cell;

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
        let instance = control.instance().to_string();
        Ok(Self {
            source: Some(source),
            control,
            instance,
            last: Cell::new(snapshot),
        })
    }

    pub fn snapshot(&self) -> (PerformanceSnapshot, bool) {
        if let Ok(value) = self.control.try_snapshot() {
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
                    consumption: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::tests::{audio_file, serial};
    use stagemaster_playback::LoopSchedule;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn fallback_keeps_the_original_consumption_instead_of_restamping_cached_position() {
        let _guard = serial();
        let (_dir, path) = audio_file(8000, 2, 80);
        let audio = PerformanceAudio::prepare(
            path,
            0,
            &LoopSchedule::new(10, vec![]).unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap();
        let (source, control) = audio.source(0, &AtomicBool::new(false)).unwrap();
        let mut voice = Voice::new(source, control).unwrap();
        assert_eq!(voice.instance, voice.control.instance().to_string());
        let source = voice.source.as_mut().unwrap();
        source.next().unwrap();
        source.next().unwrap();
        let (first, pending) = voice.snapshot();
        assert!(!pending);
        assert!(first.consumption.is_some());
        voice.control.block_snapshot_for_test();
        for _ in 0..10 {
            let (cached, pending) = voice.snapshot();
            assert!(pending);
            assert_eq!(cached.position, first.position);
            assert_eq!(cached.consumption, first.consumption);
        }
    }
}
