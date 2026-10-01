//! Converts semantic recipes into the existing scene effect representation.
use super::{EffectTemplateRecipe, IntensityWaveform};
use crate::{EffectValues, SceneEffect, Waveform};

impl EffectTemplateRecipe {
    pub(super) fn format_version(&self) -> u16 {
        match self {
            Self::IntensityWave { .. } => 1,
            Self::IntensityKeyframes { .. } => 2,
        }
    }

    pub(super) fn from_effect(effect: &SceneEffect) -> Result<Self, String> {
        if effect.target_path.is_some() || effect.channels.len() != 1 {
            return Err("请只导出单一亮度属性的效果".into());
        }
        match (&effect.waveform, &effect.channels[0]) {
            (
                Waveform::Keyframes,
                EffectValues::Keyframes {
                    attribute,
                    keyframes,
                },
            ) if attribute == "dimmer" => Ok(Self::IntensityKeyframes {
                keyframes: keyframes.clone(),
            }),
            (
                waveform,
                EffectValues::Range {
                    attribute,
                    low,
                    high,
                },
            ) if attribute == "dimmer" => {
                let waveform = match waveform {
                    Waveform::Smooth => IntensityWaveform::Smooth,
                    Waveform::Triangle => IntensityWaveform::Triangle,
                    Waveform::Pulse => IntensityWaveform::Pulse,
                    _ => return Err("当前模板文件只支持基础曲线和关键帧亮度效果".into()),
                };
                Ok(Self::IntensityWave {
                    waveform,
                    low: *low,
                    high: *high,
                    duty_percent: effect.duty_percent,
                })
            }
            _ => Err("当前模板文件只支持基础曲线和关键帧亮度效果".into()),
        }
    }

    pub(super) fn effect_values(&self) -> (Waveform, u8, Vec<EffectValues>) {
        match self {
            Self::IntensityWave {
                waveform,
                low,
                high,
                duty_percent,
            } => (
                match waveform {
                    IntensityWaveform::Smooth => Waveform::Smooth,
                    IntensityWaveform::Triangle => Waveform::Triangle,
                    IntensityWaveform::Pulse => Waveform::Pulse,
                },
                *duty_percent,
                vec![EffectValues::Range {
                    attribute: "dimmer".into(),
                    low: *low,
                    high: *high,
                }],
            ),
            Self::IntensityKeyframes { keyframes } => (
                Waveform::Keyframes,
                50,
                vec![EffectValues::Keyframes {
                    attribute: "dimmer".into(),
                    keyframes: keyframes.clone(),
                }],
            ),
        }
    }
}
