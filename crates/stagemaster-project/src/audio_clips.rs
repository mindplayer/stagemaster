//! Explicit, non-overlapping host lighting intervals, independent of beat markers.
use crate::{AudioTimeline, Document, array};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) const CAPABILITY: &str = "media.audio-clips";
pub const MAX_LIGHTING_CLIPS: usize = 512;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioLightingClip {
    pub id: String,
    pub name: String,
    pub scene_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub fade_ms: u64,
    pub locked: bool,
    #[serde(
        default = "crate::audio_clip_state::enabled_default",
        skip_serializing_if = "crate::audio_clip_state::is_enabled"
    )]
    pub enabled: bool,
}
/// A borrowed scheduling identity; no UI, audio device or clock dependency.
#[derive(Clone, Copy, Debug)]
pub struct AudioLightingRef<'a> {
    pub id: &'a str,
    pub scene_id: &'a str,
    pub start_ms: u64,
}
impl AudioTimeline {
    #[must_use]
    pub fn lighting_at(&self, time_ms: u64) -> Option<AudioLightingRef<'_>> {
        if let Some(clips) = &self.lighting_clips {
            return clips
                .iter()
                .find(|c| c.enabled && c.start_ms <= time_ms && time_ms < c.end_ms)
                .map(|c| AudioLightingRef {
                    id: &c.id,
                    scene_id: &c.scene_id,
                    start_ms: c.start_ms,
                });
        }
        self.scene_at(time_ms).and_then(|m| {
            Some(AudioLightingRef {
                id: &m.id,
                scene_id: m.scene_id.as_deref()?,
                start_ms: m.time_ms,
            })
        })
    }
}
pub(super) fn validate(root: &Value, track: &AudioTimeline) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    let Some(clips) = &track.lighting_clips else {
        return if declared {
            Err("灯光片段能力声明缺少片段轨道".into())
        } else {
            Ok(())
        };
    };
    if !declared {
        return Err("工程缺少独立灯光片段能力声明".into());
    }
    if clips.len() > MAX_LIGHTING_CLIPS {
        return Err("灯光片段最多 512 个".into());
    }
    if track
        .markers
        .iter()
        .any(|m| m.scene_id.is_some() || m.fade_ms != 0)
    {
        return Err("独立片段模式的卡点只能作节奏标记，请使用灯光片段绑定场景".into());
    }
    let mut previous_end = 0;
    for clip in clips {
        if clip.name.trim().is_empty() {
            return Err("灯光片段名称不能为空".into());
        }
        if clip.start_ms >= clip.end_ms || clip.end_ms > track.duration_ms() {
            return Err(format!(
                "片段“{}”的结束须晚于开始，且位于音乐范围内",
                clip.name
            ));
        }
        if clip.start_ms < previous_end {
            return Err(format!(
                "片段“{}”与前一片段重叠，请调整开始或结束时间",
                clip.name
            ));
        }
        if clip.fade_ms > clip.end_ms - clip.start_ms {
            return Err(format!("片段“{}”的进入渐变不能超过片段长度", clip.name));
        }
        if !array(&root["lighting"], "scenes")
            .iter()
            .any(|s| s["id"] == clip.scene_id)
        {
            return Err(format!(
                "片段“{}”引用的灯光场景不存在，请先修改或删除片段",
                clip.name
            ));
        }
        previous_end = clip.end_ms;
    }
    Ok(())
}
impl Document {
    /// Compile a legacy marker or an explicit clip in the timeline's active mode.
    /// # Errors
    /// Rejects missing identities or unsupported scene output.
    pub fn compile_audio_lighting(
        &self,
        id: Option<&str>,
    ) -> Result<crate::CompiledSequence, String> {
        let track = self.audio_timeline().ok_or("工程没有音乐")?;
        let Some(clips) = &track.lighting_clips else {
            return self.compile_audio_marker(id);
        };
        let Some(id) = id else {
            return self.compile_audio_scene(None);
        };
        let clip = clips
            .iter()
            .find(|c| c.id == id)
            .ok_or("此灯光片段不存在")?;
        if !clip.enabled {
            return Err("此灯光片段已停用".into());
        }
        let previous = clips
            .iter()
            .find(|c| c.enabled && c.end_ms == clip.start_ms);
        self.compile_audio_entry(
            &clip.scene_id,
            clip.fade_ms,
            previous.map(|p| (p.scene_id.as_str(), p.end_ms - p.start_ms)),
        )
    }
}
