//! Compile authoring effects against each step's resolved static target.
use crate::{EffectValues, SceneEffect, Transition, Waveform};
use serde_json::Value;
use stagemaster_playback::{Curve, EffectChannel, Keyframe};

pub(super) fn compile(
    root: &Value,
    effects: &[SceneEffect],
    targets: &[(String, String)],
    values: &[u16],
) -> Result<Vec<EffectChannel>, String> {
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
            let phase = u64::from(effect.phase_degrees) * 65_536 / 360
                + u64::from(effect.spread_degrees) * order * 65_536 / (360 * count);
            let mut world_frames = effect
                .target_path
                .as_ref()
                .map(|path| {
                    let mut previous = [0; 2];
                    for (axis, key) in ["pan", "tilt"].iter().enumerate() {
                        let index = targets
                            .iter()
                            .position(|(id, attribute)| id == fixture && attribute == key)
                            .ok_or("轨迹属性未纳入计划")?;
                        previous[axis] = values[index];
                    }
                    crate::world_line::compile(root, fixture, path, previous)
                })
                .transpose()
                .map_err(|reason| format!("效果“{}”：{reason}", effect.name))?;
            for channel in &effect.channels {
                let index = targets
                    .iter()
                    .position(|(id, key)| id == fixture && key == channel.attribute())
                    .ok_or("效果属性未纳入计划")?;
                let (low, high) = match channel {
                    EffectValues::Range { low, high, .. } => (*low, *high),
                    _ => (0, 0),
                };
                let axis_phase = match channel {
                    EffectValues::Position { phase_degrees, .. } => {
                        u64::from(*phase_degrees) * 65_536 / 360
                    }
                    _ => 0,
                };
                let curve = match effect.waveform {
                    Waveform::Smooth => Curve::Smooth,
                    Waveform::Triangle => Curve::Triangle,
                    Waveform::Pulse => Curve::Pulse,
                    Waveform::Keyframes => keyframes(channel)?,
                    Waveform::Position => Curve::Keyframes(
                        crate::position_effect::compile(root, fixture, channel, values[index])
                            .map_err(|reason| format!("效果“{}”：{reason}", effect.name))?,
                    ),
                    Waveform::WorldLine => Curve::Keyframes(std::mem::take(
                        &mut world_frames.as_mut().ok_or("空间轨迹缺失")?
                            [usize::from(channel.attribute() == "tilt")],
                    )),
                };
                compiled.push(EffectChannel {
                    index,
                    low,
                    high,
                    period_ms: effect.period_ms,
                    phase: u16::try_from((phase + axis_phase) % 65_536)
                        .expect("phase is within one turn"),
                    curve,
                    duty_percent: effect.duty_percent,
                });
            }
        }
    }
    Ok(compiled)
}

fn keyframes(channel: &EffectValues) -> Result<Curve, String> {
    Ok(Curve::Keyframes(
        channel
            .frames()
            .ok_or("关键帧缺失")?
            .iter()
            .map(|frame| Keyframe {
                phase: u16::try_from((u32::from(frame.position) * 65_536).div_ceil(10_000))
                    .expect("validated position"),
                value: frame.value,
                transition: match frame.transition {
                    Transition::Hold => stagemaster_playback::Transition::Hold,
                    Transition::Linear => stagemaster_playback::Transition::Linear,
                    Transition::Smooth => stagemaster_playback::Transition::Smooth,
                },
            })
            .collect(),
    ))
}
