//! Deterministic lighting entry transitions on a host music timeline.
use crate::{AudioTimeline, CompiledSequence, Document, array};
use serde_json::{Value, json};
use stagemaster_playback::{Plan, Player};

pub(super) const CAPABILITY: &str = "media.audio-transitions";
pub(super) fn validate(root: &Value, track: &AudioTimeline) -> Result<(), String> {
    let mut present = false;
    for marker in &track.markers {
        if marker.fade_ms == 0 {
            continue;
        }
        present = true;
        let end = track
            .markers
            .iter()
            .find(|next| next.scene_id.is_some() && next.time_ms > marker.time_ms)
            .map_or(track.duration_ms(), |next| next.time_ms);
        if marker.scene_id.is_none() || marker.fade_ms > end - marker.time_ms {
            return Err(format!(
                "卡点“{}”的渐变须绑定场景，且不能超过下一灯光段落或音乐结束（最多 {} 毫秒）",
                marker.name,
                end - marker.time_ms
            ));
        }
    }
    if present
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY && r["version"] == 1)
    {
        return Err("工程缺少音乐灯光渐变能力声明".into());
    }
    Ok(())
}

impl Document {
    /// Compile one lighting segment with the same entry snapshot after any seek.
    /// Previous dynamic output is sampled at the boundary; only incoming effects continue.
    /// # Errors
    /// Rejects missing music/markers and unsupported scene/output capacity.
    pub fn compile_audio_marker(
        &self,
        marker_id: Option<&str>,
    ) -> Result<CompiledSequence, String> {
        let track = self.audio_timeline().ok_or("工程没有音乐")?;
        let Some(id) = marker_id else {
            return self.compile_audio_scene(None);
        };
        let marker = track
            .markers
            .iter()
            .find(|m| m.id == id && m.scene_id.is_some())
            .ok_or("此灯光卡点不存在")?;
        let mut compiled = self.compile_audio_scene(marker.scene_id.as_deref())?;
        if marker.fade_ms == 0 {
            return Ok(compiled);
        }
        let previous = track
            .markers
            .iter()
            .rev()
            .find(|m| m.time_ms < marker.time_ms && m.scene_id.is_some());
        let from = if let Some(previous) = previous {
            let source = self.compile_audio_scene(previous.scene_id.as_deref())?;
            let mut player = Player::new(source.plan, 0);
            player.execute(0, 0)?;
            // Previous entry fade is complete here because validated transitions never overlap.
            player.advance(marker.time_ms - previous.time_ms)?;
            player.values().to_vec()
        } else {
            compiled.plan.defaults().to_vec()
        };
        let mut steps = compiled.plan.steps().to_vec();
        steps[0].fade_ms = marker.fade_ms;
        compiled.plan = Plan::with_snap_attributes(
            from,
            steps,
            false,
            compiled.plan.effects().to_vec(),
            compiled.plan.snap_attributes().to_vec(),
        )?;
        Ok(compiled)
    }
}

impl Document {
    /// Compile an isolated scene or the fixture defaults for the start of a music track.
    /// # Errors
    /// Reject invalid scene references, patch conflicts or core playback capacity violations.
    pub fn compile_audio_scene(
        &self,
        scene_id: Option<&str>,
    ) -> Result<crate::CompiledSequence, String> {
        if let Some(id) = scene_id {
            return self.compile_scene(id);
        }
        let mut temporary = self.clone();
        let id = crate::id();
        temporary.root["lighting"]["scenes"]
            .as_array_mut()
            .ok_or("没有灯光场景库")?
            .push(json!({"id":id,"name":"音乐起始默认值","assignments":[]}));
        temporary.compile_scene(&id)
    }
}
