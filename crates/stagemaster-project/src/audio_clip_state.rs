//! Capability boundary for optional clip playback state.
use crate::{AudioTimeline, array};
use serde_json::{Value, json};
pub(super) const CAPABILITY: &str = "media.audio-clip-state";
pub(super) const fn enabled_default() -> bool {
    true
}
#[allow(clippy::trivially_copy_pass_by_ref)] // Serde skip_serializing_if takes a reference.
pub(super) const fn is_enabled(value: &bool) -> bool {
    *value
}
pub(super) fn validate(root: &Value, track: Option<&AudioTimeline>) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    let clips = track.and_then(|t| t.lighting_clips.as_ref());
    if declared && clips.is_none() {
        return Err("片段启停能力声明缺少独立片段轨道".into());
    }
    if clips.is_some_and(|c| c.iter().any(|c| !c.enabled)) && !declared {
        return Err("工程缺少灯光片段启停能力声明".into());
    }
    Ok(())
}
pub(super) fn declare_if_needed(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if track
        .lighting_clips
        .as_ref()
        .is_some_and(|clips| clips.iter().any(|c| !c.enabled))
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
