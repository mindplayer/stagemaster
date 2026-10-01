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
            });
        }
        AudioEdit::PutLightingClip { clip } => {
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
            *previous = clip;
        }
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
        })
        .collect();
    track.lighting_clips = Some(clips);
    for marker in &mut track.markers {
        marker.scene_id = None;
        marker.fade_ms = 0;
    }
    Ok(())
}
