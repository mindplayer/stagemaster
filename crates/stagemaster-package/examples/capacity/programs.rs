use stagemaster_package::{Mapping, Output, Program, StepLabel};
use stagemaster_playback::{Curve, EffectChannel, Keyframe, Plan, Step, Transition};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Coarse,
    Fine,
    Steps,
    Keyframes,
}
impl Shape {
    pub const ALL: [Self; 4] = [Self::Coarse, Self::Fine, Self::Steps, Self::Keyframes];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Coarse => "满路粗调",
            Self::Fine => "完整双字节",
            Self::Steps => "128 步密集切换",
            Self::Keyframes => "128 路关键帧效果",
        }
    }
    pub const fn max_scale(self) -> usize {
        match self {
            Self::Coarse | Self::Fine => 128,
            Self::Steps => 512,
            Self::Keyframes => 32,
        }
    }
    pub const fn min_scale(self) -> usize {
        if matches!(self, Self::Keyframes) {
            2
        } else {
            1
        }
    }
}
fn value(index: usize, step: usize) -> u16 {
    u16::try_from((index * 977 + step * 253 + 1024) % 65536).unwrap()
}
fn effect(index: usize, frames: usize) -> EffectChannel {
    let curve = if frames == 0 {
        match index % 3 {
            0 => Curve::Smooth,
            1 => Curve::Triangle,
            _ => Curve::Pulse,
        }
    } else {
        Curve::Keyframes(
            (0..frames)
                .map(|n| Keyframe {
                    phase: u16::try_from(n * 65536 / frames).unwrap(),
                    value: value(index, n),
                    transition: match n % 3 {
                        0 => Transition::Smooth,
                        1 => Transition::Linear,
                        _ => Transition::Hold,
                    },
                })
                .collect(),
        )
    };
    EffectChannel {
        index,
        low: 0,
        high: 65535,
        period_ms: 1237,
        phase: u16::try_from(index * 511).unwrap(),
        curve,
        duty_percent: 37,
    }
}
pub fn program(shape: Shape, scale: usize) -> Program {
    let (attributes, steps, frames, fine) = match shape {
        Shape::Coarse => (512, scale, 0, false),
        Shape::Fine => (256, scale, 0, true),
        Shape::Steps => (scale, 128, 0, false),
        Shape::Keyframes => (128, 1, scale, false),
    };
    let effects = (0..steps)
        .map(|_| {
            if matches!(shape, Shape::Steps) {
                Vec::new()
            } else {
                (0..128).map(|i| effect(i, frames)).collect()
            }
        })
        .collect();
    let plan = Plan::with_effects(
        (0..attributes).map(|i| value(i, 0)).collect(),
        (0..steps)
            .map(|n| Step {
                target: (0..attributes).map(|i| value(i, n + 1)).collect(),
                delay_ms: 0,
                fade_ms: if matches!(shape, Shape::Steps) { 0 } else { 20 },
                wait_ms: if matches!(shape, Shape::Keyframes) {
                    None
                } else {
                    Some(1)
                },
            })
            .collect(),
        !matches!(shape, Shape::Keyframes),
        effects,
    )
    .unwrap();
    Program {
        plan,
        output: Output {
            universe: 1,
            mappings: (0..attributes)
                .map(|i| Mapping {
                    coarse: u16::try_from(i * if fine { 2 } else { 1 } + 1).unwrap(),
                    fine: fine.then(|| u16::try_from(i * 2 + 2).unwrap()),
                })
                .collect(),
        },
        labels: (0..steps)
            .map(|n| StepLabel {
                id: (n as u128 + 1).to_be_bytes(),
                name: format!("边界步骤 {n:03}"),
                number: (n + 1).to_string(),
            })
            .collect(),
    }
}
