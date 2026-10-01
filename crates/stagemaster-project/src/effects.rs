//! Scene-owned authoring effects. Compilation is separate from editing and presentation.
use crate::{array, editing, id, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneEffect {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub fixture_ids: Vec<String>,
    pub period_ms: u32,
    pub spread_degrees: u16,
    pub phase_degrees: u16,
    pub reverse: bool,
    pub waveform: Waveform,
    pub duty_percent: u8,
    pub channels: Vec<EffectValues>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_path: Option<Box<crate::world_line::TargetPath>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_source: Option<Box<crate::EffectTemplateSource>>,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Waveform {
    Smooth,
    Triangle,
    Pulse,
    Keyframes,
    Position,
    WorldLine,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum EffectValues {
    Target {
        attribute: String,
    },
    Range {
        attribute: String,
        low: u16,
        high: u16,
    },
    Position {
        attribute: String,
        #[serde(rename = "amplitudeDegrees")]
        amplitude_degrees: String,
        #[serde(rename = "offsetDegrees")]
        offset_degrees: String,
        #[serde(rename = "phaseDegrees")]
        phase_degrees: u16,
    },
    Keyframes {
        attribute: String,
        keyframes: Vec<EffectKeyframe>,
    },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectKeyframe {
    pub position: u16,
    pub value: u16,
    pub transition: Transition,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Transition {
    Hold,
    Linear,
    Smooth,
}
impl EffectValues {
    pub(super) fn attribute(&self) -> &str {
        match self {
            Self::Range { attribute, .. }
            | Self::Target { attribute }
            | Self::Keyframes { attribute, .. }
            | Self::Position { attribute, .. } => attribute,
        }
    }
    pub(super) fn frames(&self) -> Option<&[EffectKeyframe]> {
        match self {
            Self::Keyframes { keyframes, .. } => Some(keyframes),
            Self::Range { .. } | Self::Position { .. } | Self::Target { .. } => None,
        }
    }
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum EffectEdit {
    Put {
        scene_id: String,
        effect: SceneEffect,
    },
    Remove {
        scene_id: String,
        id: String,
    },
}

pub(super) fn apply(root: &mut Value, command: EffectEdit) -> Result<(), String> {
    let scene_id = match &command {
        EffectEdit::Put { scene_id, .. } | EffectEdit::Remove { scene_id, .. } => scene_id,
    };
    let scene = editing::find(editing::list(root, "scenes")?, scene_id)?;
    if scene.get("effects").is_none() {
        scene["effects"] = json!([]);
    }
    let effects = scene["effects"].as_array_mut().ok_or("场景效果字段无效")?;
    match command {
        EffectEdit::Put { effect, .. } => {
            let is_keyframes = matches!(effect.waveform, Waveform::Keyframes);
            let encoded = serde_json::to_value(&effect).map_err(|_| "效果编码失败")?;
            if let Some(existing) = effects.iter_mut().find(|e| e["id"] == effect.id) {
                if existing.get("templateSource") != encoded.get("templateSource") {
                    return Err("已有灯效的模板来源快照不能被改写或移除".into());
                }
                *existing = encoded;
            } else {
                effects.push(encoded);
            }
            let requires = root["requires"].as_array_mut().ok_or("缺少工程能力声明")?;
            for key in [
                Some("lighting.effects.basic"),
                effect
                    .template_source
                    .is_some()
                    .then_some(crate::effect_template::CAPABILITY),
                is_keyframes.then_some("lighting.effects.keyframes"),
                matches!(effect.waveform, Waveform::Position)
                    .then_some("lighting.effects.position"),
                matches!(effect.waveform, Waveform::WorldLine)
                    .then_some(crate::world_line::CAPABILITY),
            ]
            .into_iter()
            .flatten()
            {
                if !requires.iter().any(|r| r["key"] == key) {
                    requires.push(json!({"key":key,"version":1}));
                }
            }
        }
        EffectEdit::Remove { id, .. } => editing::remove(effects, &id)?,
    }
    Ok(())
}

pub(super) fn read(scene: &Value) -> Vec<SceneEffect> {
    array(scene, "effects")
        .iter()
        .map(|value| serde_json::from_value(value.clone()).expect("validated effect"))
        .collect()
}

pub(super) fn renew_ids(scene: &mut Value) {
    if let Some(effects) = scene.get_mut("effects").and_then(Value::as_array_mut) {
        for effect in effects {
            effect["id"] = id().into();
        }
    }
}

pub(super) fn validate(root: &Value) -> Result<(), String> {
    for scene in array(&root["lighting"], "scenes") {
        if !array(scene, "effects").is_empty()
            && !array(root, "requires")
                .iter()
                .any(|r| r["key"] == "lighting.effects.basic")
        {
            return Err("动态效果缺少工程能力声明".into());
        }
        let mut occupied = BTreeSet::new();
        for effect in read(scene) {
            validate_keyframes(root, &effect)?;
            crate::position_effect::validate(root, &effect)?;
            crate::world_line::validate(root, &effect)?;
            for fixture in &effect.fixture_ids {
                for channel in &effect.channels {
                    editing::validate_target(root, fixture, channel.attribute())
                        .map_err(|reason| format!("效果“{}”：{reason}", effect.name))?;
                    if effect.enabled
                        && !occupied.insert((fixture.clone(), channel.attribute().to_owned()))
                    {
                        return Err(format!(
                            "场景“{}”的效果“{}”与其他已启用效果控制了同一灯具属性，请先停用或调整灯具范围",
                            text(scene, "name"),
                            effect.name
                        ));
                    }
                }
            }
            let unique: BTreeSet<_> = effect
                .channels
                .iter()
                .map(EffectValues::attribute)
                .collect();
            if unique.len() != effect.channels.len() {
                return Err(format!("效果“{}”的属性重复", effect.name));
            }
        }
    }
    Ok(())
}

pub(super) fn keyframe_count(effects: &[SceneEffect]) -> usize {
    effects
        .iter()
        .filter(|effect| effect.enabled)
        .map(|effect| {
            effect.fixture_ids.len()
                * effect
                    .channels
                    .iter()
                    .map(|channel| match channel {
                        EffectValues::Position { .. } => crate::position_effect::SAMPLES,
                        EffectValues::Target { .. } => crate::world_line_curve::SAMPLES,
                        _ => channel.frames().map_or(0, <[EffectKeyframe]>::len),
                    })
                    .sum::<usize>()
        })
        .sum()
}

fn validate_keyframes(root: &Value, effect: &SceneEffect) -> Result<(), String> {
    let enabled = matches!(effect.waveform, Waveform::Keyframes);
    if enabled
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == "lighting.effects.keyframes")
    {
        return Err("关键帧效果缺少工程能力声明".into());
    }
    let reference = effect.channels.first().and_then(EffectValues::frames);
    for channel in &effect.channels {
        if enabled != channel.frames().is_some() {
            return Err("效果变化方式与属性关键帧不一致".into());
        }
        if let Some(frames) = channel.frames() {
            if frames[0].position != 0 || frames.windows(2).any(|p| p[0].position >= p[1].position)
            {
                return Err(format!(
                    "效果“{}”的关键帧须从 0% 开始并按时间递增",
                    effect.name
                ));
            }
            let reference = reference.ok_or("效果关键帧缺失")?;
            if reference.len() != frames.len()
                || reference
                    .iter()
                    .zip(frames)
                    .any(|(a, b)| a.position != b.position || a.transition != b.transition)
            {
                return Err("同一效果各属性的关键帧位置与过渡方式须一致".into());
            }
        }
    }
    Ok(())
}
