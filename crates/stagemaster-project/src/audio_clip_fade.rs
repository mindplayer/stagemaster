//! Historical continuous start values keyed by identity, never DMX addresses.
use crate::{AudioLightingClip, AudioTimeline, Document, array, audio::MAX_AUDIO_MS};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(super) const CAPABILITY: &str = "media.audio-clip-fade";
const MAX_SNAPSHOT_VALUES: usize = 32_768;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipEntryFade {
    pub duration_ms: u64,
    pub offset_ms: u64,
    pub from: Vec<ClipFadeValue>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipFadeValue {
    pub fixture_id: String,
    pub attribute: String,
    pub value: u16,
}
impl ClipEntryFade {
    #[must_use]
    pub fn visible_ms(&self, length: u64) -> u64 {
        self.duration_ms.saturating_sub(self.offset_ms).min(length)
    }
}

pub(super) fn validate(root: &Value, track: Option<&AudioTimeline>) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|require| require["key"] == CAPABILITY && require["version"] == 1);
    let clips = track.and_then(|track| track.lighting_clips.as_ref());
    if declared && clips.is_none() {
        return Err("保留渐变能力声明缺少独立片段轨道".into());
    }
    let mut total = 0;
    for clip in clips.into_iter().flatten() {
        let Some(fade) = &clip.entry_fade else {
            continue;
        };
        let reason = validate_fade(root, clip, fade, declared);
        if let Err(reason) = reason {
            return Err(format!("片段“{}”的保留渐变：{reason}", clip.name));
        }
        total += fade.from.len();
        if total > MAX_SNAPSHOT_VALUES {
            return Err("保留渐变的起始值总数超过 32768 项，请减少片段或重新设置渐变".into());
        }
    }
    Ok(())
}
pub(super) fn validate_fade(
    root: &Value,
    clip: &AudioLightingClip,
    fade: &ClipEntryFade,
    declared: bool,
) -> Result<(), String> {
    if !declared {
        return Err("缺少 media.audio-clip-fade 能力声明".into());
    }
    if !(1..=MAX_AUDIO_MS).contains(&fade.duration_ms)
        || fade
            .offset_ms
            .checked_add(clip.end_ms.saturating_sub(clip.start_ms))
            .is_none_or(|end| end > MAX_AUDIO_MS)
        || fade.visible_ms(clip.end_ms.saturating_sub(clip.start_ms)) != clip.fade_ms
        || fade.from.len() > 512
    {
        return Err("时间、可见渐变长度或快照数量无效".into());
    }
    let mut targets = BTreeSet::new();
    for value in &fade.from {
        if !targets.insert((&value.fixture_id, &value.attribute)) {
            return Err("起始值中存在重复灯具属性".into());
        }
        let profile = crate::fixture_value::profile(&root["lighting"], &value.fixture_id)
            .map_err(|_| "起始值引用的灯具不存在，请先重新设置渐变")?;
        if !array(profile, "attributes").iter().any(|attribute| {
            attribute["key"] == value.attribute && attribute["valueType"]["kind"] == "normalized"
        }) {
            return Err("起始值引用的连续属性不存在，请先重新设置渐变".into());
        }
    }
    Ok(())
}

/// Capture only once. Subsequent cuts retain the original weights and from-values.
pub(super) fn prepare(root: &Value, track: &mut AudioTimeline, id: &str) -> Result<(), String> {
    let clips = track
        .lighting_clips
        .as_mut()
        .ok_or("请先转换为独立灯光片段")?;
    let index = clips
        .iter()
        .position(|clip| clip.id == id)
        .ok_or("此灯光片段已不存在")?;
    let clip = &clips[index];
    if clip.entry_fade.is_some() || clip.entry_crossfade.is_some() || clip.fade_ms == 0 {
        return Ok(());
    }
    if clip.locked {
        return Err("此灯光片段已锁定，请先解锁".into());
    }
    let document = Document {
        root: root.clone(),
        saved_revision: None,
    };
    if clip.fade_mode == crate::ClipFadeMode::Dynamic {
        let fade = Document::crossfade_origin(clips, clip)?;
        clips[index].entry_crossfade = Some(fade);
        return Ok(());
    }
    let compiled = document.compile_clip(clips, clip)?;
    let from = compiled
        .output
        .attribute_bindings()
        .filter(|binding| !binding.3)
        .map(|(fixture, attribute, index, _)| ClipFadeValue {
            fixture_id: fixture.into(),
            attribute: attribute.into(),
            value: compiled.plan.defaults()[index],
        })
        .collect();
    clips[index].entry_fade = Some(ClipEntryFade {
        duration_ms: clip.fade_ms,
        offset_ms: 0,
        from,
    });
    Ok(())
}
pub(super) fn declare_if_needed(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if track
        .lighting_clips
        .as_ref()
        .is_some_and(|clips| clips.iter().any(|clip| clip.entry_fade.is_some()))
        && !array(root, "requires")
            .iter()
            .any(|require| require["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":CAPABILITY,"version":1}));
    }
    Ok(())
}
