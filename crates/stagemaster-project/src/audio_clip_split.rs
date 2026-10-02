//! Atomic split including historical entry fades; no scene, music or adjacent interval mutation.
use crate::{AudioTimeline, audio::MAX_AUDIO_MS, audio_clips::MAX_LIGHTING_CLIPS};
pub(super) fn split(
    root: &serde_json::Value,
    track: &mut AudioTimeline,
    id: &str,
    time_ms: u64,
) -> Result<(), String> {
    let clips = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?;
    let index = clips
        .iter()
        .position(|c| c.id == id)
        .ok_or("此灯光片段已不存在")?;
    let source = &clips[index];
    if source.locked {
        return Err("此灯光片段已锁定，请先解锁".into());
    }
    if clips.len() >= MAX_LIGHTING_CLIPS {
        return Err("分割后超过 512 个灯光片段".into());
    }
    if time_ms <= source.start_ms || time_ms >= source.end_ms {
        return Err("分割位置必须在片段开始与结束之间".into());
    }
    let elapsed = time_ms - source.start_ms;
    if elapsed < source.fade_ms && source.entry_fade.is_none() && source.entry_crossfade.is_none() {
        crate::audio_clip_fade::prepare(root, track, id)?;
    }
    let clips = track.lighting_clips.as_mut().expect("clip track");
    let source = &clips[index];
    let mut right = source.clone();
    right.effect_offset_ms = source
        .effect_offset_ms
        .checked_add(elapsed)
        .filter(|v| *v <= MAX_AUDIO_MS)
        .ok_or("分割后的效果起点超出范围")?;
    right.id = crate::id();
    right.start_ms = time_ms;
    if let Some(fade) = &mut right.entry_crossfade {
        fade.shift(i128::from(elapsed))?;
        right.fade_ms = fade.visible_ms(right.end_ms - time_ms);
    } else if let Some(fade) = &mut right.entry_fade {
        fade.offset_ms = fade
            .offset_ms
            .checked_add(elapsed)
            .ok_or("渐变起点超出范围")?;
        right.fade_ms = fade.visible_ms(right.end_ms - time_ms);
    } else {
        right.fade_ms = 0;
    }
    if let Some(fade) = &clips[index].entry_crossfade {
        clips[index].fade_ms = fade.visible_ms(elapsed);
    }
    if let Some(fade) = &clips[index].entry_fade {
        clips[index].fade_ms = fade.visible_ms(elapsed);
    }
    clips[index].end_ms = time_ms;
    clips.insert(index + 1, right);
    Ok(())
}
pub(super) fn reset(track: &mut AudioTimeline, id: &str) -> Result<(), String> {
    let clip = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or("此灯光片段已不存在")?;
    if clip.locked {
        return Err("此灯光片段已锁定，请先解锁".into());
    }
    clip.effect_offset_ms = 0;
    Ok(())
}

pub(super) fn reset_entry(track: &mut AudioTimeline, id: &str) -> Result<(), String> {
    let clip = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?
        .iter_mut()
        .find(|clip| clip.id == id)
        .ok_or("此灯光片段已不存在")?;
    if clip.locked {
        return Err("此灯光片段已锁定，请先解锁".into());
    }
    clip.entry_fade = None;
    clip.entry_crossfade = None;
    Ok(())
}
