//! Fixed-budget lowering plus a continuous error envelope, never endpoint-only acceptance.
use crate::PositionModel;
use stagemaster_playback::{Keyframe, Transition};
use stagemaster_spatial::{positioning::JointAngles, trajectory::LineTrajectory};
use std::f64::consts::{PI, TAU};

pub(super) const SAMPLES: usize = 32;
fn progress(phase: f64) -> f64 {
    (1.0 - (TAU * phase).cos()) * 0.5
}
fn array(angles: JointAngles) -> [f64; 2] {
    [angles.pan_degrees, angles.tilt_degrees]
}

pub(super) fn compile(
    line: LineTrajectory,
    model: &PositionModel,
    fine: [bool; 2],
    distance: f64,
    tolerance: f64,
) -> Result<[Vec<Keyframe>; 2], String> {
    let axes = [&model.pan, &model.tilt];
    let mut rounding = [0.0; 2];
    for (index, axis) in axes.iter().enumerate() {
        let range = axis.range()?;
        rounding[index] = (range.max_degrees - range.min_degrees)
            * if fine[index] {
                1.0 / 65535.0
            } else {
                2.0 / 255.0
            };
    }
    let phase_bounds = line
        .derivative_bounds(0.0, 1.0)
        .map_err(|_| "轨迹区间无效")?;
    let phase_error = array(phase_bounds.first).map(|v| v * PI / 65536.0);
    let mut worst: f64 = 0.0;
    for i in 0..32 {
        let from = progress(f64::from(i) / 32.0);
        let to = progress(f64::from(i + 1) / 32.0);
        let bounds = line
            .derivative_bounds(from.min(to), from.max(to))
            .map_err(|_| "轨迹区间无效")?;
        let first = array(bounds.first);
        let second = array(bounds.second);
        let angle = (0..2)
            .map(|axis| {
                (second[axis] * PI * PI + first[axis] * 2.0 * PI * PI) / (8.0 * 32.0 * 32.0)
                    + rounding[axis]
                    + phase_error[axis]
            })
            .sum::<f64>();
        worst = worst.max(distance * angle.to_radians());
    }
    if worst > tolerance {
        return Err(format!(
            "32 帧与通道精度下的轨迹误差上界为 {worst:.3} 米，超过允许的 {tolerance:.3} 米；请缩短轨迹、远离转轴或调整允许误差"
        ));
    }
    let mut frames = [Vec::with_capacity(SAMPLES), Vec::with_capacity(SAMPLES)];
    for i in 0_u16..32 {
        let angles = array(
            line.sample(progress(f64::from(i) / 32.0))
                .map_err(|_| "轨迹采样无效")?,
        );
        for axis in 0..2 {
            frames[axis].push(Keyframe {
                phase: i * 2048,
                value: axes[axis].encode(angles[axis], fine[axis])?,
                transition: Transition::Linear,
            });
        }
    }
    Ok(frames)
}
