//! Atomic split after entry fade; no scene, music or adjacent interval mutation.
use crate::{AudioTimeline, audio::MAX_AUDIO_MS, audio_clips::MAX_LIGHTING_CLIPS};
pub(super) fn split(track: &mut AudioTimeline, id: &str, time_ms: u64) -> Result<(), String> {
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
    if elapsed < source.fade_ms {
        return Err("进入渐变尚未结束，请将分割位置移到渐变结束之后".into());
    }
    let mut right = source.clone();
    right.effect_offset_ms = source
        .effect_offset_ms
        .checked_add(elapsed)
        .filter(|v| *v <= MAX_AUDIO_MS)
        .ok_or("分割后的效果起点超出范围")?;
    right.id = crate::id();
    right.start_ms = time_ms;
    right.fade_ms = 0;
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
