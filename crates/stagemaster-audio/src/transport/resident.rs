use super::{AudioLoadRequest, OutputBinding, Transport};
use stagemaster_playback::LoopSchedule;
use std::path::PathBuf;

impl Transport {
    /// Bind an output owned by the caller. Clearing/loading media retains this explicit route.
    #[must_use]
    pub fn with_output(binding: OutputBinding) -> Self {
        Self {
            binding: Some(binding),
            ..Self::default()
        }
    }

    /// Prepare formal, observable playback, including ordinary music with no authored repeats.
    /// # Errors
    /// Reject invalid ranges or mismatched loop schedules, without touching the current voice.
    pub fn load_performance_request(
        &self,
        file: PathBuf,
        in_ms: u64,
        out_ms: u64,
        schedule: Option<LoopSchedule>,
    ) -> Result<AudioLoadRequest, String> {
        let duration = out_ms.checked_sub(in_ms).ok_or("音乐范围无效")?;
        let schedule = match schedule {
            Some(schedule) => schedule,
            None => LoopSchedule::new(duration, Vec::new())?,
        };
        self.load_request(file, in_ms, out_ms, Some(schedule))
    }

    /// Attach a prepared paused source and let actual callbacks render silent frames.
    /// No material is consumed. The caller must observe health before coordinating execution.
    /// # Errors
    /// Reject ordinary audition, playing/ended/failed voices and unavailable output.
    pub fn prime_performance(&mut self) -> Result<(), String> {
        let voice = self
            .performance
            .as_ref()
            .and_then(|p| p.voice.as_ref())
            .ok_or("请先准备正式音源")?;
        let snapshot = voice.control.try_snapshot()?;
        if voice.control.requested_playback().playing
            || snapshot.stopped
            || snapshot.position.ended
            || snapshot.problem.is_some()
        {
            return Err("音源须处于已准备的暂停状态".into());
        }
        if self.output_failed() {
            return Err("音频输出已中断，请重新准备输出设备".into());
        }
        if self.player.is_some() {
            return Ok(());
        }
        voice
            .source
            .as_ref()
            .ok_or("演出音源未准备")?
            .check_ready()?;
        let player = self.new_player()?;
        let voice = self
            .performance
            .as_mut()
            .and_then(|p| p.voice.as_mut())
            .ok_or("演出音源未准备")?;
        voice
            .source
            .as_ref()
            .ok_or("演出音源未准备")?
            .check_ready()?;
        player.append(voice.source.take().ok_or("演出音源未准备")?);
        player.play(); // Pull paused zeros; never Player::pause, which stops health observations.
        self.player = Some(player);
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
}
