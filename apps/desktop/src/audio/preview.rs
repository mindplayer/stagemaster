//! Lighting follows the native audio cursor. UI polling never supplies elapsed time.
use stagemaster_audio::{Position, Transport};
use stagemaster_playback::Player;
use stagemaster_project::{AudioTimeline, CompiledOutput, Document};
use std::path::PathBuf;
#[derive(Default)]
pub(crate) struct AudioPreview {
    pub transport: Transport,
    track: Option<AudioTimeline>,
    lighting: Option<Lighting>,
}
struct Lighting {
    key: (u64, Option<String>, u64),
    elapsed: u64,
    player: Player,
    output: CompiledOutput,
}
impl AudioPreview {
    pub fn load(&mut self, path: PathBuf, track: AudioTimeline) -> Result<(), String> {
        self.transport.load(path, track.in_ms, track.out_ms)?;
        self.track = Some(track);
        self.lighting = None;
        Ok(())
    }
    pub fn clear(&mut self) {
        self.transport.clear();
        self.track = None;
        self.lighting = None;
    }
    pub fn active(&self) -> bool {
        self.track.is_some()
    }
    pub fn synchronize(&mut self, doc: &Document) {
        match (&mut self.track, doc.audio_timeline()) {
            (Some(current), Some(next))
                if current.asset == next.asset
                    && current.in_ms == next.in_ms
                    && current.out_ms == next.out_ms =>
            {
                *current = next;
                self.lighting = None;
            }
            _ => self.clear(),
        }
    }
    pub fn position(&self) -> Position {
        self.transport.position()
    }
    pub fn render(
        &mut self,
        doc: &Document,
        version: u64,
        master: stagemaster_playback::OutputMaster,
    ) -> Result<crate::preview::RenderOutput, String> {
        let track = self.track.as_ref().ok_or("请先准备音乐")?;
        let pos = self.transport.position();
        let marker = track.lighting_at(pos.position_ms);
        let origin = marker.map_or(0, |m| m.start_ms);
        let identity = marker.map(|m| m.id.to_owned());
        let elapsed = pos.position_ms.saturating_sub(origin);
        let key = (version, identity, origin);
        if self
            .lighting
            .as_ref()
            .is_none_or(|l| l.key != key || elapsed < l.elapsed)
        {
            if doc.view().fixtures.is_empty() {
                return Ok(crate::preview::RenderOutput {
                    status: "unloaded",
                    output: None,
                });
            }
            let compiled = doc.compile_audio_lighting(marker.map(|m| m.id))?;
            let mut player = Player::new(compiled.plan, 0);
            player.execute(0, 0)?;
            self.lighting = Some(Lighting {
                key,
                elapsed: 0,
                player,
                output: compiled.output,
            });
        }
        let lighting = self.lighting.as_mut().ok_or("灯光试听未就绪")?;
        lighting.player.advance(elapsed)?;
        lighting.elapsed = elapsed;
        Ok(crate::preview::RenderOutput {
            status: if pos.playing { "running" } else { "paused" },
            output: Some(
                lighting
                    .output
                    .render_with_master(lighting.player.values(), master)?,
            ),
        })
    }
}
