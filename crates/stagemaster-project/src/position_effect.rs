//! Physical relative movement is lowered once; runtime and renderers share the plan.
use crate::{EffectValues, SceneEffect, Waveform, array, position, text};
use serde_json::Value;
use stagemaster_playback::{Keyframe, Transition};

pub(super) const SAMPLES: usize = 32;

pub(super) fn validate(root: &Value, effect: &SceneEffect) -> Result<(), String> {
    let motion = matches!(effect.waveform, Waveform::Position);
    if motion
        && !array(root, "requires")
            .iter()
            .any(|c| c["key"] == "lighting.effects.position")
    {
        return Err("位置效果缺少工程能力声明".into());
    }
    for channel in &effect.channels {
        if motion != matches!(channel, EffectValues::Position { .. }) {
            return Err("位置效果与属性参数不一致".into());
        }
        if let EffectValues::Position {
            amplitude_degrees,
            offset_degrees,
            ..
        } = channel
        {
            crate::stage::decimal(amplitude_degrees, 0.0, 3600.0)?;
            crate::stage::decimal(offset_degrees, -3600.0, 3600.0)?;
        }
    }
    if motion {
        for id in &effect.fixture_ids {
            let (fixture, profile) = position::fixture_profile(root, id)?;
            if position::model(profile)?.is_none() {
                return Err(format!(
                    "效果“{}”：灯具“{}”未定义两轴运动模型",
                    effect.name,
                    text(fixture, "name")
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn compile(
    root: &Value,
    fixture_id: &str,
    channel: &EffectValues,
    base: u16,
) -> Result<Vec<Keyframe>, String> {
    let EffectValues::Position {
        attribute,
        amplitude_degrees,
        offset_degrees,
        ..
    } = channel
    else {
        return Err("缺少位置效果参数".into());
    };
    let (fixture, profile) = position::fixture_profile(root, fixture_id)?;
    let model = position::model(profile)?.ok_or("未定义两轴运动模型")?;
    let axis = match attribute.as_str() {
        "pan" => model.pan,
        "tilt" => model.tilt,
        _ => return Err("位置效果只能控制水平或垂直轴".into()),
    };
    let fine = position::fine(profile, attribute);
    let amplitude = crate::stage::decimal(amplitude_degrees, 0.0, 3600.0)?;
    let center = axis.decode(base, fine)? + crate::stage::decimal(offset_degrees, -3600.0, 3600.0)?;
    // Check the entire continuous stroke, not only sampled points; never silently clip.
    if axis.encode(center - amplitude, fine).is_err()
        || axis.encode(center + amplitude, fine).is_err()
    {
        return Err(format!(
            "灯具“{}”{}运动超出机械行程 {}–{}°；请减小幅度、调整中心偏移或静态位置",
            text(fixture, "name"),
            if attribute == "pan" {
                "水平"
            } else {
                "垂直"
            },
            axis.min_degrees,
            axis.max_degrees
        ));
    }
    (0_u16..32)
        .map(|i| {
            Ok(Keyframe {
                phase: i * 2048,
                value: axis.encode(
                    center + amplitude * (f64::from(i) * std::f64::consts::TAU / 32.0).sin(),
                    fine,
                )?,
                transition: Transition::Linear,
            })
        })
        .collect()
}
