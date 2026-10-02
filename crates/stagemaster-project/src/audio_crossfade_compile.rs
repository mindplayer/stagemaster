//! At most two independent sources; never recursively walks the predecessor chain.
use crate::{
    AudioLightingClip, AudioSegmentPlan, ClipCrossfadeSource, ClipEntryCrossfade, ClipFadeMode,
    CompiledAudioSegment, Document,
};
use stagemaster_playback::{CrossfadeTiming, Player};
impl Document {
    /// Compile either timeline mode into a host sampler and one shared output mapping.
    /// # Errors
    /// Rejects missing/disabled clips, incompatible mappings, budgets and overlapping transitions.
    pub fn compile_audio_segment(&self, id: Option<&str>) -> Result<CompiledAudioSegment, String> {
        let track = self.audio_timeline().ok_or("工程没有音乐")?;
        if let (Some(clips), Some(id)) = (&track.lighting_clips, id) {
            let clip = clips
                .iter()
                .find(|c| c.id == id)
                .ok_or("此灯光片段不存在")?;
            if !clip.enabled {
                return Err("此灯光片段已停用".into());
            }
            if clip.fade_mode == ClipFadeMode::Dynamic
                && (clip.fade_ms > 0 || clip.entry_crossfade.is_some())
            {
                let fade = Self::crossfade_origin(clips, clip)?;
                return self.compile_crossfade(clip, &fade);
            }
        }
        let compiled = self.compile_audio_lighting(id)?;
        Ok(CompiledAudioSegment {
            playback: AudioSegmentPlan::Single(compiled.plan),
            output: compiled.output,
        })
    }
    pub(super) fn crossfade_origin(
        clips: &[AudioLightingClip],
        clip: &AudioLightingClip,
    ) -> Result<ClipEntryCrossfade, String> {
        if let Some(fade) = &clip.entry_crossfade {
            return Ok(fade.clone());
        }
        let source = if let Some(previous) = clips
            .iter()
            .find(|p| p.enabled && p.end_ms == clip.start_ms)
        {
            if previous.entry_crossfade.as_ref().is_some_and(|fade| {
                fade.offset_ms + previous.end_ms - previous.start_ms < fade.duration_ms
            }) {
                return Err(format!(
                    "片段“{}”的动态交叉尚未结束，不能作为“{}”的新交叉来源",
                    previous.name, clip.name
                ));
            }
            ClipCrossfadeSource {
                scene_id: Some(previous.scene_id.clone()),
                effect_offset_ms: previous.effect_offset_ms,
                elapsed_ms: previous.end_ms - previous.start_ms,
                entry_fade: previous.entry_fade.clone(),
            }
        } else {
            ClipCrossfadeSource {
                scene_id: None,
                effect_offset_ms: 0,
                elapsed_ms: 0,
                entry_fade: None,
            }
        };
        Ok(ClipEntryCrossfade {
            duration_ms: clip.fade_ms,
            offset_ms: 0,
            source,
        })
    }
    pub(super) fn compile_crossfade(
        &self,
        clip: &AudioLightingClip,
        fade: &ClipEntryCrossfade,
    ) -> Result<CompiledAudioSegment, String> {
        let mut source = self.compile_audio_scene(fade.source.scene_id.as_deref())?;
        if let Some(entry) = &fade.source.entry_fade {
            source.plan = crate::audio_clip_compile::historical_plan(&source, entry)?;
        }
        source.plan = source
            .plan
            .with_effect_time_offset(fade.source.effect_offset_ms)?;
        let mut target = self.compile_audio_scene(Some(&clip.scene_id))?;
        target.plan = target.plan.with_effect_time_offset(clip.effect_offset_ms)?;
        if !source
            .output
            .attribute_bindings()
            .eq(target.output.attribute_bindings())
        {
            return Err("交叉来源与目标的灯具属性绑定不一致".into());
        }
        Ok(CompiledAudioSegment {
            playback: AudioSegmentPlan::Crossfade {
                source: source.plan,
                target: target.plan,
                timing: CrossfadeTiming {
                    duration_ms: fade.duration_ms,
                    offset_ms: fade.offset_ms,
                    source_elapsed_ms: fade.source.elapsed_ms,
                    target_elapsed_ms: 0,
                },
            },
            output: target.output,
        })
    }
    pub(super) fn clip_boundary(&self, previous: &AudioLightingClip) -> Result<Vec<u16>, String> {
        let elapsed = previous.end_ms - previous.start_ms;
        if let Some(fade) = &previous.entry_crossfade {
            let mut player = self
                .compile_crossfade(previous, fade)?
                .playback
                .into_player()?;
            player.advance(elapsed)?;
            return Ok(player.values().to_vec());
        }
        // Ordinary fades of either kind are complete at this boundary.
        let mut source = self.compile_audio_scene(Some(&previous.scene_id))?;
        if let Some(fade) = &previous.entry_fade {
            source.plan = crate::audio_clip_compile::historical_plan(&source, fade)?;
        }
        source.plan = source
            .plan
            .with_effect_time_offset(previous.effect_offset_ms)?;
        let mut player = Player::new(source.plan, 0);
        player.execute(0, 0)?;
        player.advance(elapsed)?;
        Ok(player.values().to_vec())
    }
}
