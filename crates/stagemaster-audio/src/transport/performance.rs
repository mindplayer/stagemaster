use super::{
    Transport,
    preparation::{
        AudioLoadRequest, AudioLoadTicket, AudioSeekRequest, Identity, PreparedAudioLoad,
        PreparedAudioSeek,
    },
    voice::Performance,
};
use stagemaster_playback::LoopSchedule;
use std::{path::PathBuf, sync::Arc};

impl Transport {
    fn identity(&self) -> Identity {
        Identity {
            owner: self.owner.clone(),
            revision: self.revision,
        }
    }

    fn check_identity(&self, identity: &Identity) -> Result<(), String> {
        if !Arc::ptr_eq(&self.owner, &identity.owner) || self.revision != identity.revision {
            return Err("播放会话已变化，准备结果未应用".into());
        }
        Ok(())
    }

    /// # Errors
    /// Reject invalid media ranges or a schedule with a different local duration.
    pub fn load_request(
        &self,
        file: PathBuf,
        in_ms: u64,
        out_ms: u64,
        schedule: Option<LoopSchedule>,
    ) -> Result<AudioLoadRequest, String> {
        self.load_ticket().request(file, in_ms, out_ms, schedule)
    }

    #[must_use]
    pub fn load_ticket(&self) -> AudioLoadTicket {
        AudioLoadTicket {
            identity: self.identity(),
        }
    }

    /// # Errors
    /// Reject stale or failed preparation without clearing the previous usable source.
    pub fn apply_load(&mut self, prepared: PreparedAudioLoad) -> Result<(), String> {
        self.check_identity(&prepared.identity)?;
        if let Some((_, voice)) = &prepared.performance {
            voice
                .source
                .as_ref()
                .ok_or("演出音源未准备")?
                .check_ready()?;
        }
        self.clear();
        self.file = Some(prepared.file);
        self.in_ms = prepared.in_ms;
        self.duration_ms = prepared.duration_ms;
        self.performance = prepared
            .performance
            .map(|(audio, voice)| Performance::new(audio, voice));
        Ok(())
    }

    /// Return work requiring preparation outside the session lock; an existing paused voice resumes in place.
    /// # Errors
    /// Reject inconsistent audio observations.
    pub fn play_preparation(&self) -> Result<Option<AudioSeekRequest>, String> {
        let Some(performance) = &self.performance else {
            return Ok(None);
        };
        let position = self.position();
        if position.problem.is_none()
            && !position.performance.as_ref().is_some_and(|p| p.ended)
            && let Some(voice) = &performance.voice
        {
            if voice
                .source
                .as_ref()
                .is_some_and(|s| s.check_ready().is_err())
            {
                return self.seek_preparation(position.position_ms, true);
            }
            if voice.source.is_some() || self.player.as_ref().is_some_and(|p| !p.empty()) {
                return Ok(None);
            }
        }
        let target = if position.position_ms >= self.duration_ms {
            0
        } else {
            position.position_ms
        };
        self.seek_preparation(target, true)
    }

    /// # Errors
    /// Reject an out-of-range target; linear playback returns no separate preparation.
    pub fn seek_preparation(
        &self,
        position_ms: u64,
        playing: bool,
    ) -> Result<Option<AudioSeekRequest>, String> {
        if position_ms > self.duration_ms {
            return Err("播放位置超出音乐范围".into());
        }
        Ok(self
            .performance
            .as_ref()
            .map(|performance| AudioSeekRequest {
                identity: self.identity(),
                audio: performance.audio.clone(),
                position_ms,
                playing: playing && position_ms < self.duration_ms,
            }))
    }

    /// # Errors
    /// Reject stale preparation or output failure while retaining the prior voice.
    pub fn apply_seek(&mut self, mut prepared: PreparedAudioSeek) -> Result<(), String> {
        self.check_identity(&prepared.identity)?;
        if self.performance.is_none() {
            return Err("正式演出音频未载入".into());
        }
        prepared
            .voice
            .source
            .as_ref()
            .ok_or("演出音源未准备")?
            .check_ready()?;
        let player = if prepared.playing {
            let player = self.new_player()?;
            prepared
                .voice
                .source
                .as_ref()
                .ok_or("演出音源未准备")?
                .check_ready()?;
            prepared.voice.control.request_playback(true)?;
            player.append(prepared.voice.source.take().ok_or("演出音源未准备")?);
            Some(player)
        } else {
            None
        };
        let state = self.performance.as_mut().ok_or("正式演出音频未载入")?;
        state.voice = Some(prepared.voice);
        self.player = player;
        self.base_ms = prepared.position_ms;
        self.revision = self.revision.wrapping_add(1);
        if let Some(player) = &self.player {
            player.play();
        }
        Ok(())
    }

    /// # Errors
    /// Reject stale run/pass/region controls, ended sources and source failures.
    pub fn exit_performance(
        &mut self,
        instance: &str,
        region: usize,
        pass: u64,
        requested: bool,
    ) -> Result<(), String> {
        let voice = self
            .performance
            .as_ref()
            .and_then(|p| p.voice.as_ref())
            .ok_or("当前没有可控制的演出循环")?;
        if voice.instance != instance {
            return Err("播放实例已变化，退出操作未执行".into());
        }
        voice.control.request_exit(region, pass, requested)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    pub(super) fn play_performance(&mut self) -> Result<(), String> {
        if self.play_preparation()?.is_some() {
            return Err("演出播放须先完成音源准备".into());
        }
        if let Some(player) = &self.player {
            self.performance
                .as_ref()
                .and_then(|p| p.voice.as_ref())
                .ok_or("演出音源未准备")?
                .control
                .request_playback(true)?;
            player.play();
        } else {
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
            voice.control.request_playback(true)?;
            player.append(voice.source.take().ok_or("演出音源未准备")?);
            self.player = Some(player);
            self.player.as_ref().ok_or("音频设备未就绪")?.play();
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
}
