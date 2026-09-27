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
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Waveform {
    Smooth,
    Triangle,
    Pulse,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectValues {
    pub attribute: String,
    pub low: u16,
    pub high: u16,
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
            let encoded = serde_json::to_value(&effect).map_err(|_| "效果编码失败")?;
            if let Some(existing) = effects.iter_mut().find(|e| e["id"] == effect.id) {
                *existing = encoded;
            } else {
                effects.push(encoded);
            }
            let requires = root["requires"].as_array_mut().ok_or("缺少工程能力声明")?;
            if !requires
                .iter()
                .any(|r| r["key"] == "lighting.effects.basic")
            {
                requires.push(json!({"key":"lighting.effects.basic","version":1}));
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
            for fixture in &effect.fixture_ids {
                for channel in &effect.channels {
                    editing::validate_target(root, fixture, &channel.attribute)
                        .map_err(|reason| format!("效果“{}”：{reason}", effect.name))?;
                    if effect.enabled
                        && !occupied.insert((fixture.clone(), channel.attribute.clone()))
                    {
                        return Err(format!(
                            "场景“{}”的效果“{}”与其他已启用效果控制了同一灯具属性，请先停用或调整灯具范围",
                            text(scene, "name"),
                            effect.name
                        ));
                    }
                }
            }
            let unique: BTreeSet<_> = effect.channels.iter().map(|c| &c.attribute).collect();
            if unique.len() != effect.channels.len() {
                return Err(format!("效果“{}”的属性重复", effect.name));
            }
        }
    }
    Ok(())
}

pub(super) fn compile(
    effects: &[SceneEffect],
    targets: &[(String, String)],
) -> Result<Vec<stagemaster_playback::EffectChannel>, String> {
    let mut compiled = Vec::new();
    for effect in effects.iter().filter(|e| e.enabled) {
        let count = u64::try_from(effect.fixture_ids.len()).map_err(|_| "效果灯具超限")?;
        for (index, fixture) in effect.fixture_ids.iter().enumerate() {
            let order = if effect.reverse {
                effect.fixture_ids.len() - 1 - index
            } else {
                index
            };
            let order = u64::try_from(order).map_err(|_| "效果灯具超限")?;
            let phase = (u64::from(effect.phase_degrees) * 65_536 / 360
                + u64::from(effect.spread_degrees) * order * 65_536 / (360 * count))
                % 65_536;
            for channel in &effect.channels {
                compiled.push(stagemaster_playback::EffectChannel {
                    index: targets
                        .iter()
                        .position(|(id, key)| id == fixture && key == &channel.attribute)
                        .ok_or("效果属性未纳入计划")?,
                    low: channel.low,
                    high: channel.high,
                    period_ms: effect.period_ms,
                    phase: u16::try_from(phase).expect("phase is within one turn"),
                    curve: match effect.waveform {
                        Waveform::Smooth => stagemaster_playback::Curve::Smooth,
                        Waveform::Triangle => stagemaster_playback::Curve::Triangle,
                        Waveform::Pulse => stagemaster_playback::Curve::Pulse,
                    },
                    duty_percent: effect.duty_percent,
                });
            }
        }
    }
    Ok(compiled)
}
