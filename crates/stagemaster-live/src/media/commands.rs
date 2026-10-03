use super::{GroupKey, Prepared, Sample, Status};
use crate::{Command, Session};
use stagemaster_time::Mapping;

impl Session {
    /// Atomically admit one local media group's prepared start or seek.
    /// Swap old players into the borrowed preparation for off-scheduler disposal on ALL paths.
    /// Successful activation invalidates its key. No device-start guarantee is implied.
    /// # Errors
    /// Invalid input leaves execution unchanged; internal errors invalidate the complete frame.
    pub fn activate_media(
        &mut self,
        prepared: &mut Prepared,
        sample: Sample,
        mapping: &Mapping,
        now_ms: u64,
    ) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.media_index(prepared.key)?;
        if now_ms >= prepared.deadline_ms
            || sample.position_ms != prepared.position_ms
            || sample.playing != prepared.playing
        {
            return Err("同步组准备已过期或实际媒体位置不匹配".into());
        }
        let generation = prepared
            .key
            .generation
            .checked_add(1)
            .ok_or("同步组播放代次已耗尽")?;
        let observed =
            self.media[index].validate(sample, mapping, self.host_clock(), now_ms, true)?;
        self.tick(now_ms)?;
        for (source, player) in &mut prepared.players {
            let entry = &mut self.sources[*source];
            let Some(current) = entry.player.as_mut() else {
                return self.finish(Err("同步组缺少播放器".into()));
            };
            std::mem::swap(current, player);
            entry.sampled_at_ms = now_ms;
            entry.reassert_at_ms = Some(now_ms);
        }
        self.media[index].generation = generation;
        self.media[index].accept(sample, observed);
        let result = self.compose(now_ms);
        self.finish(result)
    }

    /// Follow an actual media observation. No extrapolation, storage reads or plan rebuilding.
    /// # Errors
    /// Reject old generations, stale/invalid samples or backwards host time before mutation.
    pub fn observe_media(
        &mut self,
        key: GroupKey,
        sample: Sample,
        mapping: &Mapping,
        now_ms: u64,
    ) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.media_index(key)?;
        let observed =
            self.media[index].validate(sample, mapping, self.host_clock(), now_ms, false)?;
        if self.media[index].members.iter().any(|&source| {
            self.sources[source]
                .player
                .as_ref()
                .is_none_or(|p| !p.supports_position(sample.position_ms))
        }) {
            return Err("媒体位置超出来源时长".into());
        }
        self.tick(now_ms)?;
        let result = self.advance_media(index, sample, now_ms).and_then(|()| {
            self.media[index].accept(sample, observed);
            self.compose(now_ms)
        });
        self.finish(result)
    }
    fn advance_media(&mut self, index: usize, sample: Sample, now_ms: u64) -> Result<(), String> {
        let previous = self.media[index].last.ok_or("同步组缺少上次观测")?;
        for &source in &self.media[index].members {
            let entry = &mut self.sources[source];
            let player = entry.player.as_mut().ok_or("同步组缺少播放器")?;
            // Resume at the OLD position, so the first new consumption delta is not discarded.
            if !previous.playing {
                player.apply_timeline(Command::Resume, previous.position_ms)?;
            }
            player.apply_timeline(Command::Advance, sample.position_ms)?;
            if !sample.playing {
                player.apply_timeline(Command::Pause, sample.position_ms)?;
            }
            entry.sampled_at_ms = now_ms;
        }
        Ok(())
    }

    /// Release only this media group and invalidate any queued observations/preparations.
    /// # Errors
    /// Reject stale keys or time; execution failures invalidate the complete frame.
    pub fn stop_media(&mut self, key: GroupKey, now_ms: u64) -> Result<(), String> {
        self.ready(now_ms)?;
        let index = self.media_index(key)?;
        let generation = key
            .generation
            .checked_add(1)
            .ok_or("同步组播放代次已耗尽")?;
        self.tick(now_ms)?;
        let result = self.release_media(index).and_then(|()| {
            self.media[index].generation = generation;
            self.media[index].status = Status::Stopped;
            self.media[index].last = None;
            self.compose(now_ms)
        });
        self.finish(result)
    }
    fn release_media(&mut self, index: usize) -> Result<(), String> {
        if self.media[index].status == Status::Stopped {
            return Ok(());
        }
        let position = self.media[index]
            .last
            .map_or(self.now_ms, |s| s.position_ms);
        for &source in &self.media[index].members {
            self.sources[source]
                .player
                .as_mut()
                .ok_or("同步组缺少播放器")?
                .apply_timeline(Command::Stop, position)?;
        }
        Ok(())
    }
}
