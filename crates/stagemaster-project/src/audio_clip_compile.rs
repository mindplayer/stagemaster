//! Bounded compilation of historical entry fades; no predecessor recursion or extra clocks.
use crate::{AudioLightingClip, ClipEntryFade, CompiledSequence, Document};
use stagemaster_playback::Plan;

impl Document {
    pub(super) fn compile_clip(
        &self,
        clips: &[AudioLightingClip],
        clip: &AudioLightingClip,
    ) -> Result<CompiledSequence, String> {
        if clip.fade_mode == crate::ClipFadeMode::Dynamic
            && (clip.fade_ms > 0 || clip.entry_crossfade.is_some())
        {
            return Err("动态交叉须使用宿主复合采样，不能编译为设备单计划".into());
        }
        if let Some(fade) = &clip.entry_fade {
            return self.compile_clip_slice(clip, fade);
        }
        let mut compiled = self.compile_audio_scene(Some(&clip.scene_id))?;
        if clip.fade_ms > 0 {
            let from = if let Some(previous) = clips
                .iter()
                .find(|previous| previous.enabled && previous.end_ms == clip.start_ms)
            {
                self.clip_boundary(previous)?
            } else {
                compiled.plan.defaults().to_vec()
            };
            compiled.plan = entry_plan(&compiled.plan, from, clip.fade_ms, 0)?;
        }
        compiled.plan = compiled
            .plan
            .with_effect_time_offset(clip.effect_offset_ms)?;
        Ok(compiled)
    }

    fn compile_clip_slice(
        &self,
        clip: &AudioLightingClip,
        fade: &ClipEntryFade,
    ) -> Result<CompiledSequence, String> {
        let mut compiled = self.compile_audio_scene(Some(&clip.scene_id))?;
        compiled.plan =
            historical_plan(&compiled, fade)?.with_effect_time_offset(clip.effect_offset_ms)?;
        Ok(compiled)
    }
}
pub(super) fn historical_plan(
    compiled: &CompiledSequence,
    fade: &ClipEntryFade,
) -> Result<Plan, String> {
    let mut from = compiled.plan.defaults().to_vec();
    for (fixture, attribute, index, discrete) in compiled.output.attribute_bindings() {
        if !discrete
            && let Some(value) = fade
                .from
                .iter()
                .find(|value| value.fixture_id == fixture && value.attribute == attribute)
        {
            from[index] = value.value;
        }
    }
    entry_plan(&compiled.plan, from, fade.duration_ms, fade.offset_ms)
}

fn entry_plan(plan: &Plan, from: Vec<u16>, duration: u64, offset: u64) -> Result<Plan, String> {
    let mut steps = plan.steps().to_vec();
    steps[0].fade_ms = duration;
    Plan::with_snap_attributes(
        from,
        steps,
        false,
        plan.effects().to_vec(),
        plan.snap_attributes().to_vec(),
    )?
    .with_entry_fade_offset(offset)
}
