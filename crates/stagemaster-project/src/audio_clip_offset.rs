//! Explicit source-clock offset for independently scheduled lighting clips.
use crate::{AudioTimeline, array, audio::MAX_AUDIO_MS};
use serde_json::{Value, json};
pub(super) const CAPABILITY: &str = "media.audio-clip-offset";
#[allow(clippy::trivially_copy_pass_by_ref)] // Serde requires a borrowed field.
pub(super) const fn is_zero(value: &u64) -> bool {
    *value == 0
}
pub(super) fn validate(root: &Value, track: Option<&AudioTimeline>) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    let clips = track.and_then(|t| t.lighting_clips.as_ref());
    if declared && clips.is_none() {
        return Err("效果起点能力声明缺少独立片段轨道".into());
    }
    for c in clips.into_iter().flatten() {
        if c.effect_offset_ms != 0 && !declared {
            return Err("工程缺少片段效果起点能力声明".into());
        }
        if c.effect_offset_ms
            .checked_add(c.end_ms - c.start_ms)
            .is_none_or(|end| end > MAX_AUDIO_MS)
        {
            return Err(format!(
                "片段“{}”的效果起点与长度之和不能超过 3600 秒",
                c.name
            ));
        }
    }
    Ok(())
}
pub(super) fn declare_if_needed(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if track
        .lighting_clips
        .as_ref()
        .is_some_and(|clips| clips.iter().any(|c| c.effect_offset_ms != 0))
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":CAPABILITY,"version":1}));
    }
    Ok(())
}
