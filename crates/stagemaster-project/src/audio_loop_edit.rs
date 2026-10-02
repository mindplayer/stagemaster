//! Atomic authoring of named loop sections; no playback, device or clock ownership.
use crate::{AudioLoopPlays, AudioLoopRegion, AudioTimeline, MAX_AUDIO_LOOP_REGIONS};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AudioLoopEdit {
    Add {
        name: String,
        start_ms: u64,
        end_ms: u64,
        plays: AudioLoopPlays,
    },
    Put {
        region: AudioLoopRegion,
    },
    Edit {
        ids: Vec<String>,
        action: AudioLoopGroupAction,
    },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AudioLoopGroupAction {
    Move { destination_ms: u64 },
    Copy { destination_ms: u64 },
    Remove {},
    Enabled { enabled: bool },
    Locked { locked: bool },
    Plays { plays: AudioLoopPlays },
}

pub(super) fn apply(track: &mut AudioTimeline, command: AudioLoopEdit) -> Result<(), String> {
    match command {
        AudioLoopEdit::Add {
            name,
            start_ms,
            end_ms,
            plays,
        } => {
            if track.loop_regions.len() >= MAX_AUDIO_LOOP_REGIONS {
                return Err("演出循环最多 128 个区段".into());
            }
            track.loop_regions.push(AudioLoopRegion {
                id: crate::id(),
                name,
                start_ms,
                end_ms,
                plays,
                enabled: true,
                locked: false,
            });
        }
        AudioLoopEdit::Put { region } => {
            let previous = track
                .loop_regions
                .iter_mut()
                .find(|r| r.id == region.id)
                .ok_or("此循环区段已不存在")?;
            if previous.locked {
                return Err("此循环区段已锁定，请先解锁".into());
            }
            if region.locked != previous.locked || region.enabled != previous.enabled {
                return Err("请使用区段锁定或启停操作修改状态".into());
            }
            *previous = region;
        }
        AudioLoopEdit::Edit { ids, action } => edit_group(track, &ids, action)?,
    }
    track.loop_regions.sort_by_key(|r| r.start_ms);
    crate::audio_loops::validate_regions(track)
}

fn edit_group(
    track: &mut AudioTimeline,
    ids: &[String],
    action: AudioLoopGroupAction,
) -> Result<(), String> {
    let duration = track.duration_ms();
    let regions = &mut track.loop_regions;
    let selected: BTreeSet<_> = ids.iter().collect();
    if ids.is_empty() || ids.len() > MAX_AUDIO_LOOP_REGIONS || selected.len() != ids.len() {
        return Err("请选择 1–128 个不重复的循环区段".into());
    }
    let sources: Vec<_> = regions
        .iter()
        .filter(|r| selected.contains(&r.id))
        .cloned()
        .collect();
    if sources.len() != ids.len() {
        return Err("选中的循环区段已不存在，请重新选择".into());
    }
    let copying = matches!(action, AudioLoopGroupAction::Copy { .. });
    let setting_lock = matches!(action, AudioLoopGroupAction::Locked { .. });
    if !copying
        && !setting_lock
        && let Some(locked) = sources.iter().find(|r| r.locked)
    {
        return Err(format!("区段“{}”已锁定，请先解锁或移出选择", locked.name));
    }
    let destination = match action {
        AudioLoopGroupAction::Enabled { enabled } => {
            for r in regions.iter_mut().filter(|r| selected.contains(&r.id)) {
                r.enabled = enabled;
            }
            return Ok(());
        }
        AudioLoopGroupAction::Locked { locked } => {
            for r in regions.iter_mut().filter(|r| selected.contains(&r.id)) {
                r.locked = locked;
            }
            return Ok(());
        }
        AudioLoopGroupAction::Plays { plays } => {
            for r in regions.iter_mut().filter(|r| selected.contains(&r.id)) {
                r.plays = plays;
            }
            return Ok(());
        }
        AudioLoopGroupAction::Remove {} => {
            regions.retain(|r| !selected.contains(&r.id));
            return Ok(());
        }
        AudioLoopGroupAction::Move { destination_ms }
        | AudioLoopGroupAction::Copy { destination_ms } => destination_ms,
    };
    if copying && regions.len() + sources.len() > MAX_AUDIO_LOOP_REGIONS {
        return Err("复制后超过 128 个循环区段，请减少选择".into());
    }
    let first = sources[0].start_ms;
    let mut replacements = Vec::with_capacity(sources.len());
    for mut region in sources {
        let start = destination
            .checked_add(region.start_ms - first)
            .ok_or("目标时间超出音乐范围")?;
        let end = start
            .checked_add(region.end_ms - region.start_ms)
            .filter(|&end| end <= duration)
            .ok_or("移动或复制后的循环区段超出音乐范围")?;
        if let Some(conflict) = regions.iter().find(|r| {
            (copying || !selected.contains(&r.id)) && start < r.end_ms && end > r.start_ms
        }) {
            return Err(format!(
                "区段“{}”的目标与“{}”重叠，请调整目标起点",
                region.name, conflict.name
            ));
        }
        if copying {
            region.id = crate::id();
            region.locked = false;
        }
        region.start_ms = start;
        region.end_ms = end;
        replacements.push(region);
    }
    if !copying {
        regions.retain(|r| !selected.contains(&r.id));
    }
    regions.extend(replacements);
    Ok(())
}
