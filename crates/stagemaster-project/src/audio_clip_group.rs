//! Bounded, atomic authoring of independent lighting intervals.
use crate::{AudioTimeline, audio_clips::MAX_LIGHTING_CLIPS};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum LightingClipGroupAction {
    Move { destination_ms: u64 },
    Copy { destination_ms: u64 },
    Remove {},
    Enabled { enabled: bool },
    Fade { fade_ms: u64 },
}

pub(super) fn apply(
    track: &mut AudioTimeline,
    ids: &[String],
    action: LightingClipGroupAction,
) -> Result<(), String> {
    let duration = track.duration_ms();
    let clips = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?;
    let selected: BTreeSet<_> = ids.iter().collect();
    if ids.is_empty() || ids.len() > MAX_LIGHTING_CLIPS || selected.len() != ids.len() {
        return Err("请选择 1–512 个不重复的灯光片段".into());
    }
    let sources: Vec<_> = clips
        .iter()
        .filter(|c| selected.contains(&c.id))
        .cloned()
        .collect();
    if sources.len() != ids.len() {
        return Err("选中的灯光片段已不存在，请重新选择".into());
    }
    let copying = matches!(action, LightingClipGroupAction::Copy { .. });
    if !copying && let Some(locked) = sources.iter().find(|c| c.locked) {
        return Err(format!("片段“{}”已锁定，请先解锁或移出选择", locked.name));
    }
    let destination = match action {
        LightingClipGroupAction::Fade { fade_ms } => {
            if let Some(short) = sources.iter().find(|c| fade_ms > c.end_ms - c.start_ms) {
                return Err(format!(
                    "进入渐变超出片段“{}”的长度，请缩短渐变",
                    short.name
                ));
            }
            for clip in clips.iter_mut().filter(|c| selected.contains(&c.id)) {
                clip.fade_ms = fade_ms;
                clip.entry_fade = None;
            }
            return Ok(());
        }
        LightingClipGroupAction::Enabled { enabled } => {
            for clip in clips.iter_mut().filter(|c| selected.contains(&c.id)) {
                clip.enabled = enabled;
            }
            return Ok(());
        }
        LightingClipGroupAction::Remove {} => {
            clips.retain(|c| !selected.contains(&c.id));
            return Ok(());
        }
        LightingClipGroupAction::Move { destination_ms }
        | LightingClipGroupAction::Copy { destination_ms } => destination_ms,
    };
    if copying && clips.len() + sources.len() > MAX_LIGHTING_CLIPS {
        return Err("复制后超过 512 个灯光片段，请减少选择".into());
    }
    let first = sources[0].start_ms;
    let mut replacements = Vec::with_capacity(sources.len());
    for mut clip in sources {
        let start = destination
            .checked_add(clip.start_ms - first)
            .ok_or("目标时间超出音乐范围")?;
        let end = start
            .checked_add(clip.end_ms - clip.start_ms)
            .filter(|end| *end <= duration)
            .ok_or("移动或复制后的片段超出音乐范围，请调整目标起点")?;
        if let Some(conflict) = clips.iter().find(|c| {
            (copying || !selected.contains(&c.id)) && start < c.end_ms && end > c.start_ms
        }) {
            return Err(format!(
                "片段“{}”的目标与“{}”重叠，请调整目标起点",
                clip.name, conflict.name
            ));
        }
        if copying {
            clip.id = crate::id();
            clip.locked = false;
        }
        clip.start_ms = start;
        clip.end_ms = end;
        replacements.push(clip);
    }
    if !copying {
        clips.retain(|c| !selected.contains(&c.id));
    }
    clips.extend(replacements);
    clips.sort_by_key(|c| c.start_ms);
    Ok(())
}
