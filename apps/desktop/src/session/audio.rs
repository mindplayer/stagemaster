use super::Session;
use crate::audio::Command;
use stagemaster_audio::Position;
use stagemaster_project::AudioTimeline;
impl Session {
    pub(crate) fn load_audio(
        &mut self,
        generation: u32,
        path: std::path::PathBuf,
        track: AudioTimeline,
    ) -> Result<(), String> {
        self.guard(generation)?;
        self.preview.clear();
        self.audio.load(path, track)?;
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
            )
        } else {
            self.preview
                .render_output(self.content_version, self.preview.now())
        }
    }
}
