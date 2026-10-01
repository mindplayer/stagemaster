//! Authoring operations for a bounded independent lighting lane.
use crate::{AudioEdit, AudioLightingClip, AudioTimeline, audio_clips::MAX_LIGHTING_CLIPS};

pub(super) fn apply(track: &mut AudioTimeline, command: AudioEdit) -> Result<(), String> {
    if matches!(command, AudioEdit::ConvertLightingClips) {
        return convert(track);
    }
    let duration = track.duration_ms();
    let clips = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?;
    match command {
        AudioEdit::AddLightingClip {
            name,
            scene_id,
            start_ms,
            end_ms,
            fade_ms,
        } => {
            if clips.len() >= MAX_LIGHTING_CLIPS {
                return Err("灯光片段最多 512 个".into());
            }
            clips.push(AudioLightingClip {
                id: crate::id(),
                name,
                scene_id,
                start_ms,
                end_ms,
                fade_ms,
                locked: false,
                enabled: true,
                effect_offset_ms: 0,
                entry_fade: None,
            });
        }
        AudioEdit::PutLightingClip { clip } => replace(clips, clip, false)?,
        AudioEdit::TrimLightingClip { clip } => replace(clips, clip, true)?,
        AudioEdit::CopyLightingClip { id, start_ms } => {
            if clips.len() >= MAX_LIGHTING_CLIPS {
                return Err("复制后超过 512 个灯光片段".into());
            }
            let mut copy = clips
                .iter()
                .find(|c| c.id == id)
                .ok_or("此灯光片段已不存在")?
                .clone();
            let length = copy.end_ms - copy.start_ms;
            copy.end_ms = start_ms
                .checked_add(length)
                .filter(|end| *end <= duration)
                .ok_or("复制后的片段超出音乐范围")?;
            copy.start_ms = start_ms;
            copy.id = crate::id();
            copy.locked = false;
            clips.push(copy);
        }
        AudioEdit::RemoveLightingClip { id } => {
            let index = clips
                .iter()
                .position(|c| c.id == id)
                .ok_or("此灯光片段已不存在")?;
            if clips[index].locked {
                return Err("此灯光片段已锁定，请先解锁".into());
            }
            clips.remove(index);
        }
        AudioEdit::SetLightingClipLock { id, locked } => {
            clips
                .iter_mut()
                .find(|c| c.id == id)
                .ok_or("此灯光片段已不存在")?
                .locked = locked;
        }
        _ => unreachable!("only clip commands are routed here"),
    }
    clips.sort_by_key(|c| c.start_ms);
    Ok(())
}

fn convert(track: &mut AudioTimeline) -> Result<(), String> {
    if track.lighting_clips.is_some() {
        return Err("当前已是独立片段模式".into());
    }
    let bound: Vec<_> = track
        .markers
        .iter()
        .filter(|m| m.scene_id.is_some())
        .collect();
    let clips = bound
        .iter()
        .enumerate()
        .map(|(i, m)| AudioLightingClip {
            id: crate::id(),
            name: m.name.clone(),
            scene_id: m.scene_id.clone().expect("bound marker"),
            start_ms: m.time_ms,
            end_ms: bound.get(i + 1).map_or(track.duration_ms(), |n| n.time_ms),
            fade_ms: m.fade_ms,
            locked: false,
            enabled: true,
            effect_offset_ms: 0,
            entry_fade: None,
        })
        .collect();
    track.lighting_clips = Some(clips);
    for marker in &mut track.markers {
        marker.scene_id = None;
        marker.fade_ms = 0;
    }
    Ok(())
}

fn replace(
    clips: &mut [AudioLightingClip],
    mut clip: AudioLightingClip,
    trim: bool,
) -> Result<(), String> {
    let previous = clips
        .iter_mut()
        .find(|c| c.id == clip.id)
        .ok_or("此灯光片段已不存在")?;
    if previous.locked {
        return Err("此灯光片段已锁定，请先解锁".into());
    }
    if clip.locked != previous.locked {
        return Err("请使用片段锁定操作修改锁状态".into());
    }
    if clip.enabled != previous.enabled {
        return Err("请使用片段停用或恢复操作修改状态".into());
    }
    if clip.effect_offset_ms != previous.effect_offset_ms {
        return Err("请使用片段裁切、分割或重置效果起点操作修改源时间".into());
    }
    if clip.entry_fade != previous.entry_fade {
        return Err("请使用分割、内部截取或重新设置渐变操作修改保留渐变".into());
    }
    if clip.fade_ms != previous.fade_ms || clip.scene_id != previous.scene_id {
        clip.entry_fade = None;
    }
    if trim {
        if let Some(fade) = &mut clip.entry_fade {
            fade.offset_ms = if clip.start_ms >= previous.start_ms {
                fade.offset_ms
                    .checked_add(clip.start_ms - previous.start_ms)
            } else {
                fade.offset_ms
                    .checked_sub(previous.start_ms - clip.start_ms)
            }
            .ok_or("裁切开始不能早于原渐变零点或超出源范围")?;
        }
        clip.effect_offset_ms = if clip.start_ms >= previous.start_ms {
            previous
                .effect_offset_ms
                .checked_add(clip.start_ms - previous.start_ms)
                .ok_or("裁切后的效果起点超出范围")?
        } else {
            previous
                .effect_offset_ms
                .checked_sub(previous.start_ms - clip.start_ms)
                .ok_or("裁切开始不能早于效果源零点，请使用重新安排或移动片段")?
        };
    }
    if let Some(fade) = &clip.entry_fade {
        clip.fade_ms = fade.visible_ms(clip.end_ms.saturating_sub(clip.start_ms));
    }
    *previous = clip;
    Ok(())
}

/// First complete slice captures the original entry before shifting its source clock.
pub(super) fn slice(
    root: &serde_json::Value,
    track: &mut AudioTimeline,
    mut clip: AudioLightingClip,
) -> Result<(), String> {
    let source = track
        .lighting_clips
        .as_ref()
        .ok_or("请先转换为独立灯光片段")?
        .iter()
        .find(|source| source.id == clip.id)
        .ok_or("此灯光片段已不存在")?;
    if clip.entry_fade != source.entry_fade || clip.effect_offset_ms != source.effect_offset_ms {
        return Err("内部截取不能直接修改源时间或保留渐变".into());
    }
    if clip.fade_ms != source.fade_ms || clip.scene_id != source.scene_id {
        return Err("内部截取时请保留原场景和渐变，或使用重新安排".into());
    }
    if (clip.start_ms, clip.end_ms) != (source.start_ms, source.end_ms) {
        crate::audio_clip_fade::prepare(root, track, &clip.id)?;
        clip.entry_fade.clone_from(
            &track
                .lighting_clips
                .as_ref()
                .expect("clip track")
                .iter()
                .find(|source| source.id == clip.id)
                .expect("source clip")
                .entry_fade,
        );
    }
    apply(track, AudioEdit::TrimLightingClip { clip })
}
