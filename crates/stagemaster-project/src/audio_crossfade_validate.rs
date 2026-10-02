//! Strict format/time/reference checks, independent of patch compilability.
use crate::{
    AudioLightingClip, AudioTimeline, ClipEntryCrossfade, ClipFadeMode, Document, array,
    audio::MAX_AUDIO_MS, audio_crossfade::CAPABILITY,
};
use serde_json::{Value, json};

pub(super) fn validate(root: &Value, track: Option<&AudioTimeline>) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    let clips = track.and_then(|t| t.lighting_clips.as_ref());
    if declared && clips.is_none() {
        return Err("动态交叉能力声明缺少独立片段轨道".into());
    }
    let Some(clips) = clips else {
        return Ok(());
    };
    if !clips
        .iter()
        .any(|c| c.fade_mode == ClipFadeMode::Dynamic || c.entry_crossfade.is_some())
    {
        return Ok(());
    }
    let mut values: usize = clips
        .iter()
        .filter_map(|c| c.entry_fade.as_ref())
        .map(|f| f.from.len())
        .sum();
    for clip in clips {
        if clip.fade_mode == ClipFadeMode::Snapshot {
            if clip.entry_crossfade.is_some() {
                return Err("保留交叉只能用于动态交叉模式".into());
            }
            continue;
        }
        if !declared {
            return Err("缺少 media.audio-clip-crossfade 能力声明".into());
        }
        if clip.entry_fade.is_some() {
            return Err("动态交叉不能同时具有静态保留渐变".into());
        }
        if clip.fade_ms == 0 && clip.entry_crossfade.is_none() {
            continue;
        }
        let fade = Document::crossfade_origin(clips, clip)?;
        validate_origin(root, clip, &fade)
            .map_err(|reason| format!("片段“{}”的动态交叉：{reason}", clip.name))?;
        if let Some(entry) = &fade.source.entry_fade {
            // Only persisted copies consume the document snapshot budget.
            if clip.entry_crossfade.is_some() {
                values += entry.from.len();
            }
        }
        if values > 32_768 {
            return Err("保留渐变与交叉的起始值总数超过 32768 项".into());
        }
    }
    Ok(())
}
fn validate_origin(
    root: &Value,
    clip: &AudioLightingClip,
    fade: &ClipEntryCrossfade,
) -> Result<(), String> {
    let length = clip.end_ms - clip.start_ms;
    let source = &fade.source;
    if !(1..=MAX_AUDIO_MS).contains(&fade.duration_ms)
        || fade
            .offset_ms
            .checked_add(length)
            .is_none_or(|v| v > MAX_AUDIO_MS)
        || source
            .elapsed_ms
            .checked_add(source.effect_offset_ms)
            .and_then(|v| v.checked_add(length))
            .is_none_or(|v| v > MAX_AUDIO_MS)
        || fade.visible_ms(length) != clip.fade_ms
    {
        return Err("时间、可见渐变长度或来源范围无效（最长 3600 秒）".into());
    }
    if let Some(id) = &source.scene_id {
        if !array(&root["lighting"], "scenes")
            .iter()
            .any(|s| s["id"] == *id)
        {
            return Err("来源场景不存在，请先重新设置该片段的渐变".into());
        }
    } else if source.entry_fade.is_some() || source.effect_offset_ms != 0 {
        return Err("默认值来源不能含场景渐变或效果偏移".into());
    }
    if let Some(entry) = &source.entry_fade {
        // Reuse semantic snapshot checks, including its independent source clock.
        let mut source_clip = clip.clone();
        source_clip.start_ms = 0;
        source_clip.end_ms = source
            .elapsed_ms
            .checked_add(length)
            .ok_or("来源时间越界")?;
        source_clip.fade_ms = entry.visible_ms(source_clip.end_ms);
        crate::audio_clip_fade::validate_fade(root, &source_clip, entry, true)?;
    }
    Ok(())
}
pub(super) fn declare_if_needed(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if track
        .lighting_clips
        .as_ref()
        .is_some_and(|clips| clips.iter().any(|c| c.fade_mode == ClipFadeMode::Dynamic))
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
