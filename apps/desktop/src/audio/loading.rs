use super::AudioPreview;
use stagemaster_audio::{AudioLoadTicket, PreparedAudioLoad};
use stagemaster_project::AudioTimeline;
use std::{path::PathBuf, sync::atomic::AtomicBool};

/// Keeps the inspected track and audio identity inseparable across slow preparation.
pub(crate) struct LoadIntent {
    ticket: AudioLoadTicket,
    track: AudioTimeline,
}

pub(crate) struct PreparedLoad {
    ready: PreparedAudioLoad,
    track: AudioTimeline,
}

impl LoadIntent {
    pub fn prepare(self, path: PathBuf, cancel: &AtomicBool) -> Result<PreparedLoad, String> {
        let schedule = if self.track.loop_regions.iter().any(|r| r.enabled) {
            Some(self.track.compile_loops(1_000)?.schedule)
        } else {
            None
        };
        let ready = self
            .ticket
            .request(path, self.track.in_ms, self.track.out_ms, schedule)?
            .prepare(cancel)?;
        Ok(PreparedLoad {
            ready,
            track: self.track,
        })
    }
}

impl AudioPreview {
    pub fn load_intent(&self, track: AudioTimeline) -> LoadIntent {
        LoadIntent {
            ticket: self.transport.load_ticket(),
            track,
        }
    }

    pub fn apply_load(&mut self, prepared: PreparedLoad) -> Result<(), String> {
        self.transport.apply_load(prepared.ready)?;
        self.track = Some(prepared.track);
        self.lighting = None;
        Ok(())
    }

    pub fn exit_loop(
        &mut self,
        instance: &str,
        region_id: &str,
        pass: &str,
        requested: bool,
    ) -> Result<(), String> {
        let track = self.track.as_ref().ok_or("请先准备音乐")?;
        let region = track
            .loop_regions
            .iter()
            .filter(|r| r.enabled)
            .position(|r| r.id == region_id)
            .ok_or("循环区段已改变，请重新确认当前区段")?;
        let number = pass.parse::<u64>().map_err(|_| "播放遍数无效")?;
        if number == 0 || number.to_string() != pass {
            return Err("播放遍数无效".into());
        }
        self.transport
            .exit_performance(instance, region, number, requested)
    }
}

pub(super) fn same_playback(left: &AudioTimeline, right: &AudioTimeline) -> bool {
    left.asset == right.asset
        && left.in_ms == right.in_ms
        && left.out_ms == right.out_ms
        && left
            .loop_regions
            .iter()
            .filter(|r| r.enabled)
            .map(|r| (&r.id, r.start_ms, r.end_ms, r.plays))
            .eq(right
                .loop_regions
                .iter()
                .filter(|r| r.enabled)
                .map(|r| (&r.id, r.start_ms, r.end_ms, r.plays)))
}
