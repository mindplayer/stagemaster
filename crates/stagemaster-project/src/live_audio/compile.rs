use super::{LiveAudioTimeline, LiveSourceBudget, sampler::Sampler};
use crate::{CompiledOutput, Document};
use sha2::{Digest, Sha256};
use stagemaster_playback::Status;

impl Document {
    /// Prepare all active music lighting content before handing it to a scheduling owner.
    /// # Errors
    /// Reject missing music/fixtures, incompatible mappings, invalid segments or cumulative budgets.
    pub fn compile_live_audio(&self) -> Result<LiveAudioTimeline, String> {
        let track = self.audio_timeline().ok_or("工程没有音乐轨道")?;
        let initial = self.compile_audio_segment(None)?;
        let output = initial.output;
        let layout = output.live_layout(Sha256::digest(self.encode()?).into())?;
        let mut owned = vec![false; layout.attributes().len()];
        let mut budget = LiveSourceBudget::default();
        let defaults = Sampler::prepare(initial.playback, &mut budget, &mut owned)?;
        let mut segments = Vec::new();
        let mut ids = Vec::new();
        if let Some(clips) = &track.lighting_clips {
            for clip in clips.iter().filter(|c| c.enabled) {
                ids.push(clip.id.clone());
                self.audio_scene_footprint(&output, &clip.scene_id, &mut owned)?;
                if let Some(fade) = &clip.entry_crossfade
                    && let Some(scene) = &fade.source.scene_id
                {
                    self.audio_scene_footprint(&output, scene, &mut owned)?;
                }
                let historical = clip.entry_fade.iter().chain(
                    clip.entry_crossfade
                        .iter()
                        .flat_map(|f| &f.source.entry_fade),
                );
                for fade in historical {
                    for (fixture, key, index, _) in output.attribute_bindings() {
                        owned[index] |= fade
                            .from
                            .iter()
                            .any(|v| v.fixture_id == fixture && v.attribute == key);
                    }
                }
            }
        } else {
            for marker in &track.markers {
                if let Some(scene) = &marker.scene_id {
                    ids.push(marker.id.clone());
                    self.audio_scene_footprint(&output, scene, &mut owned)?;
                }
            }
        }
        for id in ids {
            let compiled = self.compile_audio_segment(Some(&id))?;
            if !compiled
                .output
                .attribute_bindings()
                .eq(output.attribute_bindings())
            {
                return Err("音乐灯光段落的属性映射不一致".into());
            }
            segments.push((
                id,
                Sampler::prepare(compiled.playback, &mut budget, &mut owned)?,
            ));
        }
        let count = owned.len();
        Ok(LiveAudioTimeline {
            track,
            defaults,
            segments,
            output,
            layout,
            owned,
            budget,
            active: None,
            position: 0,
            status: Status::Idle,
            claims: vec![None; count],
        })
    }
    fn audio_scene_footprint(
        &self,
        output: &CompiledOutput,
        id: &str,
        owned: &mut [bool],
    ) -> Result<(), String> {
        let view = self.view();
        let scene = view
            .scenes
            .iter()
            .find(|s| s.id == id)
            .ok_or("音乐灯光来源场景不存在")?;
        for (fixture, key, index, _) in output.attribute_bindings() {
            // This complete timeline preserves default-value gaps within its authored footprint.
            owned[index] |= scene
                .values
                .iter()
                .any(|v| v.fixture_id == fixture && v.attribute == key);
        }
        Ok(())
    }
}
