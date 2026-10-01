//! Project adaptation for world-space paths. The spatial crate owns continuous geometry.
use crate::{
    EffectValues, FixturePlacement, FixtureZero, SceneEffect, SpatialVector3, Waveform, array,
    position, text,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stagemaster_playback::Keyframe;
use stagemaster_spatial::{
    Installation,
    positioning::{Branch, JointAngles},
    trajectory::{Error, LineTrajectory},
};

pub(super) const CAPABILITY: &str = "lighting.effects.world-line";

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TargetPath {
    Line {
        from_meters: SpatialVector3,
        to_meters: SpatialVector3,
        branch: TargetBranch,
        max_error_meters: String,
    },
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetBranch {
    Auto,
    Front,
    Back,
}

pub(super) fn validate(root: &Value, effect: &SceneEffect) -> Result<(), String> {
    let world = matches!(effect.waveform, Waveform::WorldLine);
    if world != effect.target_path.is_some()
        || effect
            .channels
            .iter()
            .any(|c| world != matches!(c, EffectValues::Target { .. }))
    {
        return Err("空间轨迹与效果参数不一致".into());
    }
    let Some(TargetPath::Line {
        from_meters,
        to_meters,
        max_error_meters,
        ..
    }) = effect.target_path.as_deref()
    else {
        return Ok(());
    };
    if !array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY)
    {
        return Err("空间轨迹缺少工程能力声明".into());
    }
    if effect.channels.len() != 2
        || !["pan", "tilt"]
            .iter()
            .all(|axis| effect.channels.iter().any(|c| c.attribute() == *axis))
    {
        return Err("空间轨迹必须同时控制水平和垂直两轴".into());
    }
    let a = from_meters.numbers(100_000.0)?;
    let b = to_meters.numbers(100_000.0)?;
    if a.into_iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        < 1e-12
    {
        return Err("轨迹起点和终点不能重合".into());
    }
    crate::stage::decimal(max_error_meters, 0.001, 1.0)?;
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
    Ok(())
}

pub(super) fn compile(
    root: &Value,
    id: &str,
    path: &TargetPath,
    previous: [u16; 2],
) -> Result<[Vec<Keyframe>; 2], String> {
    let (fixture, profile) = position::fixture_profile(root, id)?;
    compile_fixture(root, id, fixture, profile, path, previous)
        .map_err(|error| format!("灯具“{}”：{error}", text(fixture, "name")))
}

fn compile_fixture(
    root: &Value,
    id: &str,
    fixture: &Value,
    profile: &Value,
    path: &TargetPath,
    previous: [u16; 2],
) -> Result<[Vec<Keyframe>; 2], String> {
    let model = position::model(profile)?.ok_or("未定义两轴运动模型")?;
    let placement = array(&root["stage"], "placements")
        .iter()
        .find(|v| v["fixtureId"] == id)
        .ok_or("尚未布置灯位，请先设置安装位置")?;
    let placement: FixturePlacement =
        serde_json::from_value(placement.clone()).map_err(|_| "灯位无效")?;
    let install = Installation {
        position_meters: placement.position_meters.numbers(100_000.0)?,
        rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0)?,
    };
    let zero: Option<FixtureZero> = fixture
        .get("zeroCorrection")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "零偏无效"))
        .transpose()?;
    let fine = [
        position::fine(profile, "pan"),
        position::fine(profile, "tilt"),
    ];
    let previous = JointAngles {
        pan_degrees: model.pan.decode(previous[0], fine[0])?,
        tilt_degrees: model.tilt.decode(previous[1], fine[1])?,
    };
    let TargetPath::Line {
        from_meters,
        to_meters,
        branch,
        max_error_meters,
    } = path;
    let from = from_meters.numbers(100_000.0)?;
    let to = to_meters.numbers(100_000.0)?;
    let branch = match branch {
        TargetBranch::Auto => None,
        TargetBranch::Front => Some(Branch::Front),
        TargetBranch::Back => Some(Branch::Back),
    };
    let line = LineTrajectory::new(
        model.head(zero.as_ref())?,
        install,
        from,
        to,
        previous,
        branch,
    )
    .map_err(|error| match error {
        Error::NearAxis => "轨迹经过或过近灯具转轴，请调整起止点",
        Error::Unreachable => "完整轨迹超出机械行程，请调整起止点或解分支",
        Error::EmptyPath => "轨迹起点和终点不能重合",
        _ => "轨迹、灯位或运动模型无效",
    })?;
    let distance = [from, to]
        .into_iter()
        .map(|point| {
            point
                .into_iter()
                .zip(install.position_meters)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(0.0, f64::max);
    crate::world_line_curve::compile(
        line,
        &model,
        fine,
        distance,
        crate::stage::decimal(max_error_meters, 0.001, 1.0)?,
    )
}
