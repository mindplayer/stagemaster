//! Portable music reference and beat markers. No file paths, decoder or output devices.
use crate::{Document, array};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_AUDIO_MS: u64 = 3_600_000;
pub const MAX_AUDIO_MARKERS: usize = 512;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioAsset {
    pub digest: String,
    pub file_name: String,
    pub extension: String,
    pub duration_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioMarker {
    pub id: String,
    pub name: String,
    pub time_ms: u64,
    pub scene_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fade_ms: u64,
}
#[allow(clippy::trivially_copy_pass_by_ref)] // Serde skip_serializing_if requires a borrowed field.
fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioTimeline {
    pub asset: AudioAsset,
    pub in_ms: u64,
    pub out_ms: u64,
    pub markers: Vec<AudioMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lighting_clips: Option<Vec<crate::AudioLightingClip>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loop_regions: Vec<crate::AudioLoopRegion>,
}
impl AudioTimeline {
    #[must_use]
    pub const fn duration_ms(&self) -> u64 {
        self.out_ms.saturating_sub(self.in_ms)
    }
    #[must_use]
    pub fn scene_at(&self, time_ms: u64) -> Option<&AudioMarker> {
        self.markers
            .iter()
            .rev()
            .find(|m| m.time_ms <= time_ms && m.scene_id.is_some())
    }
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AudioEdit {
    LoopRegions {
        command: crate::AudioLoopEdit,
    },
    SetAsset {
        asset: AudioAsset,
    },
    Trim {
        in_ms: u64,
        out_ms: u64,
    },
    PutMarker {
        marker: AudioMarker,
    },
    RemoveMarker {
        id: String,
    },
    EditMarkers {
        ids: Vec<String>,
        action: crate::MarkerGroupAction,
    },
    EditLightingClips {
        ids: Vec<String>,
        action: crate::LightingClipGroupAction,
    },
    SplitLightingClip {
        id: String,
        time_ms: u64,
    },
    ResetLightingClipEntryFade {
        id: String,
    },
    SliceLightingClip {
        clip: crate::AudioLightingClip,
    },
    ResetLightingClipEffectOffset {
        id: String,
    },
    ConvertLightingClips,
    AddLightingClip {
        #[serde(default)]
        fade_mode: crate::ClipFadeMode,
        name: String,
        scene_id: String,
        start_ms: u64,
        end_ms: u64,
        fade_ms: u64,
    },
    TrimLightingClip {
        clip: crate::AudioLightingClip,
    },
    PutLightingClip {
        clip: crate::AudioLightingClip,
    },
    CopyLightingClip {
        id: String,
        start_ms: u64,
    },
    RemoveLightingClip {
        id: String,
    },
    SetLightingClipLock {
        id: String,
        locked: bool,
    },
    Clear,
}
impl Document {
    #[must_use]
    pub fn audio_timeline(&self) -> Option<AudioTimeline> {
        read(&self.root)
    }
}
pub(super) fn read(root: &Value) -> Option<AudioTimeline> {
    root.get("media")?
        .get("audioEditing")
        .map(|value| serde_json::from_value(value.clone()).expect("validated audio timeline"))
}
fn clear(root: &mut Value) -> Result<(), String> {
    root.as_object_mut().ok_or("工程无效")?.remove("media");
    root["requires"]
        .as_array_mut()
        .ok_or("能力列表无效")?
        .retain(|r| {
            r["key"] != "media.audio-editing"
                && r["key"] != crate::audio_lighting::CAPABILITY
                && r["key"] != crate::audio_clips::CAPABILITY
                && r["key"] != crate::audio_clip_state::CAPABILITY
                && r["key"] != crate::audio_clip_offset::CAPABILITY
                && r["key"] != crate::audio_clip_fade::CAPABILITY
                && r["key"] != crate::audio_crossfade::CAPABILITY
                && r["key"] != crate::audio_loops::CAPABILITY
        });
    Ok(())
}
pub(super) fn apply(root: &mut Value, command: AudioEdit) -> Result<(), String> {
    if matches!(command, AudioEdit::Clear) {
        return clear(root);
    }
    let mut track = if let AudioEdit::SetAsset { asset } = &command {
        if read(root).is_some() {
            return Err("请先移除当前音乐，再导入另一段；移除可以撤销".into());
        }
        AudioTimeline {
            in_ms: 0,
            out_ms: asset.duration_ms,
            asset: asset.clone(),
            markers: vec![],
            lighting_clips: None,
            loop_regions: vec![],
        }
    } else {
        read(root).ok_or("请先导入音乐")?
    };
    match command {
        AudioEdit::LoopRegions { command } => crate::audio_loop_edit::apply(&mut track, command)?,
        command @ (AudioEdit::ConvertLightingClips
        | AudioEdit::AddLightingClip { .. }
        | AudioEdit::TrimLightingClip { .. }
        | AudioEdit::PutLightingClip { .. }
        | AudioEdit::CopyLightingClip { .. }
        | AudioEdit::RemoveLightingClip { .. }
        | AudioEdit::SetLightingClipLock { .. }) => {
            crate::audio_clip_edit::apply(&mut track, command)?;
        }
        AudioEdit::SplitLightingClip { id, time_ms } => {
            crate::audio_clip_split::split(root, &mut track, &id, time_ms)?;
        }
        AudioEdit::SliceLightingClip { clip } => {
            crate::audio_clip_edit::slice(root, &mut track, clip)?;
        }
        AudioEdit::ResetLightingClipEntryFade { id } => {
            crate::audio_clip_split::reset_entry(&mut track, &id)?;
        }
        AudioEdit::ResetLightingClipEffectOffset { id } => {
            crate::audio_clip_split::reset(&mut track, &id)?;
        }
        AudioEdit::SetAsset { .. } | AudioEdit::Clear => {}
        AudioEdit::Trim { in_ms, out_ms } => {
            track.in_ms = in_ms;
            track.out_ms = out_ms;
        }
        AudioEdit::PutMarker { marker } => {
            if let Some(previous) = track.markers.iter_mut().find(|m| m.id == marker.id) {
                *previous = marker;
            } else {
                track.markers.push(marker);
            }
            track.markers.sort_by_key(|m| m.time_ms);
        }
        AudioEdit::EditLightingClips { ids, action } => {
            crate::audio_clip_group::apply(&mut track, &ids, action)?;
        }
        AudioEdit::EditMarkers { ids, action } => {
            crate::audio_group::apply(&mut track, &ids, action)?;
        }
        AudioEdit::RemoveMarker { id } => {
            let index = track
                .markers
                .iter()
                .position(|m| m.id == id)
                .ok_or("此卡点已不存在")?;
            track.markers.remove(index);
        }
    }
    write_track(root, &track)
}
fn write_track(root: &mut Value, track: &AudioTimeline) -> Result<(), String> {
    if track.markers.iter().any(|marker| marker.fade_ms > 0)
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == crate::audio_lighting::CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":crate::audio_lighting::CAPABILITY,"version":1}));
    }
    if track.lighting_clips.is_some()
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == crate::audio_clips::CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":crate::audio_clips::CAPABILITY,"version":1}));
    }
    crate::audio_clip_fade::declare_if_needed(root, track)?;
    crate::audio_loops::declare_if_needed(root, track)?;
    crate::audio_crossfade_validate::declare_if_needed(root, track)?;
    crate::audio_clip_state::declare_if_needed(root, track)?;
    crate::audio_clip_offset::declare_if_needed(root, track)?;
    root["media"] = json!({"systems":[],"objects":[],"audioEditing":track});
    if !array(root, "requires")
        .iter()
        .any(|r| r["key"] == "media.audio-editing")
    {
        root["requires"]
            .as_array_mut()
            .ok_or("能力列表无效")?
            .push(json!({"key":"media.audio-editing","version":1}));
    }
    Ok(())
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let Some(media) = root.get("media") else {
        crate::audio_loops::validate(root, None)?;
        crate::audio_clip_fade::validate(root, None)?;
        crate::audio_crossfade_validate::validate(root, None)?;
        crate::audio_clip_state::validate(root, None)?;
        crate::audio_clip_offset::validate(root, None)?;
        return if array(root, "requires")
            .iter()
            .any(|r| r["key"] == crate::audio_clips::CAPABILITY)
        {
            Err("灯光片段能力声明缺少音乐轨道".into())
        } else {
            Ok(())
        };
    };
    if !array(media, "systems").is_empty()
        || !array(media, "objects").is_empty()
        || media.get("audioEditing").is_none()
    {
        return Err("当前媒体编辑仅支持本机音乐卡点；外部媒体系统尚未实现".into());
    }
    if !array(root, "requires")
        .iter()
        .any(|r| r["key"] == "media.audio-editing" && r["version"] == 1)
    {
        return Err("音乐工程缺少音频卡点能力声明".into());
    }
    let track: AudioTimeline =
        serde_json::from_value(media["audioEditing"].clone()).map_err(|_| "音频卡点字段无效")?;
    if track.in_ms >= track.out_ms
        || track.out_ms > track.asset.duration_ms
        || track.asset.duration_ms > MAX_AUDIO_MS
    {
        return Err("音乐裁切范围必须在源文件内，结束晚于开始，最长 1 小时".into());
    }
    if track.asset.file_name.trim().is_empty() || track.markers.len() > MAX_AUDIO_MARKERS {
        return Err("音乐名称不能为空，卡点最多 512 个".into());
    }
    let mut previous = None;
    for marker in &track.markers {
        if marker.name.trim().is_empty()
            || marker.time_ms >= track.duration_ms()
            || previous.is_some_and(|p| p >= marker.time_ms)
        {
            return Err("卡点名称不能为空；时间须有序、不重复，且位于音乐裁切范围内".into());
        }
        if let Some(scene) = &marker.scene_id
            && !array(&root["lighting"], "scenes")
                .iter()
                .any(|s| s["id"] == *scene)
        {
            return Err(format!(
                "卡点“{}”引用的灯光场景不存在，请先解除绑定",
                marker.name
            ));
        }
        previous = Some(marker.time_ms);
    }
    crate::audio_lighting::validate(root, &track)?;
    crate::audio_loops::validate(root, Some(&track))?;
    crate::audio_clips::validate(root, &track)?;
    crate::audio_clip_fade::validate(root, Some(&track))?;
    crate::audio_crossfade_validate::validate(root, Some(&track))?;
    crate::audio_clip_state::validate(root, Some(&track))?;
    crate::audio_clip_offset::validate(root, Some(&track))?;
    Ok(())
}
