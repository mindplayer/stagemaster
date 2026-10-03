use super::Session;
use crate::audio::Command;
use stagemaster_audio::Position;
#[cfg(test)]
use stagemaster_project::AudioTimeline;
impl Session {
    pub(crate) fn configure_audio_output(
        &mut self,
        scope: stagemaster_audio::OutputScope,
    ) -> Result<(), String> {
        self.audio.transport.set_output_scope(scope)
    }

    pub(crate) fn audio_load_intent(
        &self,
        generation: u32,
    ) -> Result<Option<crate::audio::LoadIntent>, String> {
        self.guard(generation)?;
        Ok(self
            .document
            .as_ref()
            .and_then(stagemaster_project::Document::audio_timeline)
            .map(|track| self.audio.load_intent(track)))
    }

    pub(crate) fn apply_audio_load(
        &mut self,
        generation: u32,
        prepared: crate::audio::PreparedLoad,
    ) -> Result<(), String> {
        self.guard(generation)?;
        self.audio.apply_load(prepared)?;
        self.preview.clear();
        Ok(())
    }

    pub(crate) fn audio_preparation(
        &self,
        generation: u32,
        command: &Command,
    ) -> Result<Option<stagemaster_audio::AudioSeekRequest>, String> {
        self.guard(generation)?;
        match command {
            Command::Play => self.audio.transport.play_preparation(),
            Command::Seek { position_ms } => self
                .audio
                .transport
                .seek_preparation(*position_ms, self.audio.position().playing),
            _ => Ok(None),
        }
    }

    pub(crate) fn apply_audio_preparation(
        &mut self,
        generation: u32,
        prepared: stagemaster_audio::PreparedAudioSeek,
    ) -> Result<Position, String> {
        self.guard(generation)?;
        self.audio.transport.apply_seek(prepared)?;
        self.previs_source = crate::previs::protocol::Source::Playback;
        Ok(self.audio.position())
    }
    pub(crate) fn audio_loop_request(
        &self,
        generation: u32,
        range: Option<stagemaster_audio::LoopRange>,
    ) -> Result<stagemaster_audio::LoopRequest, String> {
        self.guard(generation)?;
        self.audio.transport.loop_request(range)
    }
    pub(crate) fn apply_audio_loop(
        &mut self,
        generation: u32,
        prepared: stagemaster_audio::PreparedLoop,
    ) -> Result<Position, String> {
        self.guard(generation)?;
        self.audio.transport.apply_loop(prepared)?;
        self.previs_source = crate::previs::protocol::Source::Playback;
        Ok(self.audio.position())
    }

    #[cfg(test)]
    pub(crate) fn load_audio(
        &mut self,
        generation: u32,
        path: std::path::PathBuf,
        track: AudioTimeline,
    ) -> Result<(), String> {
        self.guard(generation)?;
        self.audio.load(path, track)?;
        self.preview.clear();
        Ok(())
    }
    pub(crate) fn audio_request(
        &mut self,
        generation: u32,
        command: Command,
    ) -> Result<Position, String> {
        self.guard(generation)?;
        match command {
            Command::Snapshot => {}
            Command::SetLoop { .. } => return Err("循环准备须由独立音频入口执行".into()),
            Command::ExitLoop {
                instance,
                region_id,
                pass,
                requested,
            } => self
                .audio
                .exit_loop(&instance, &region_id, &pass, requested)?,
            Command::Volume { percent } => self.audio.transport.set_volume(percent)?,
            Command::Play => {
                self.audio.transport.play()?;
                self.previs_source = crate::previs::protocol::Source::Playback;
            }
            Command::Pause => self.audio.transport.pause(),
            Command::Stop => self.audio.transport.stop(),
            Command::Seek { position_ms } => {
                self.audio.transport.seek(position_ms)?;
                self.previs_source = crate::previs::protocol::Source::Playback;
            }
        }
        Ok(self.audio.position())
    }
    pub(crate) fn render_playback(&mut self) -> Result<crate::preview::RenderOutput, String> {
        if self.audio.active() {
            self.audio.render(
                self.document.as_ref().ok_or("请先打开工程")?,
                self.content_version,
                self.output_control.master(),
            )
        } else {
            self.preview.render_output(
                self.content_version,
                self.preview.now(),
                self.output_control.master(),
            )
        }
    }
}
