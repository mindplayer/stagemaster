//! Atomic authoring operations on a bounded set of music markers.
use crate::{AudioTimeline, audio::MAX_AUDIO_MARKERS};
use serde::Deserialize;
use std::collections::BTreeSet;
#[derive(Clone, Copy, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MarkerGroupAction {
    Move { destination_ms: u64 },
    Copy { destination_ms: u64 },
    Remove,
}
pub(super) fn apply(
    track: &mut AudioTimeline,
    ids: &[String],
    action: MarkerGroupAction,
) -> Result<(), String> {
    let selected: BTreeSet<_> = ids.iter().collect();
    if ids.is_empty() || ids.len() > MAX_AUDIO_MARKERS || selected.len() != ids.len() {
        return Err("请选择 1–512 个不重复的卡点".into());
    }
    let sources: Vec<_> = track
        .markers
        .iter()
        .filter(|m| selected.contains(&m.id))
        .cloned()
        .collect();
    if sources.len() != ids.len() {
        return Err("选中的卡点已不存在，请重新选择".into());
    }
    if matches!(action, MarkerGroupAction::Remove) {
        track.markers.retain(|m| !selected.contains(&m.id));
        return Ok(());
    }
    let (destination, copying) = match action {
        MarkerGroupAction::Move { destination_ms } => (destination_ms, false),
        MarkerGroupAction::Copy { destination_ms } => (destination_ms, true),
        MarkerGroupAction::Remove => unreachable!(),
    };
    if copying && track.markers.len() + sources.len() > MAX_AUDIO_MARKERS {
        return Err("复制后超过 512 个卡点，请减少选择或拆分音乐".into());
    }
    let first = sources[0].time_ms;
    let mut replacements = Vec::with_capacity(sources.len());
    for mut marker in sources {
        let time = destination
            .checked_add(marker.time_ms - first)
            .filter(|t| *t < track.duration_ms())
            .ok_or("移动或复制后的卡点超出音乐范围，请调整目标时间")?;
        if let Some(conflict) = track
            .markers
            .iter()
            .find(|m| m.time_ms == time && (copying || !selected.contains(&m.id)))
        {
            return Err(format!(
                "目标 {} 毫秒与卡点“{}”重叠，请调整目标时间",
                time, conflict.name
            ));
        }
        if copying {
            marker.id = crate::id();
        }
        marker.time_ms = time;
        replacements.push(marker);
    }
    if !copying {
        track.markers.retain(|m| !selected.contains(&m.id));
    }
    track.markers.extend(replacements);
    track.markers.sort_by_key(|m| m.time_ms);
    Ok(())
}
