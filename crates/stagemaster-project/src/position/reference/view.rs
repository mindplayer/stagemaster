use super::{PositionReference, ReferencePoint, matches_profile, read};
use crate::{FixturePlacement, FixtureZero, array, text};
use serde::Serialize;
use serde_json::Value;
use stagemaster_spatial::{Installation, positioning::JointAngles};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionReferenceView {
    pub profile_id: String,
    pub profile_revision: String,
    pub compatible: bool,
    pub points: Vec<ReferencePointView>,
}
#[derive(Serialize)]
pub struct ReferencePointView {
    pub point: ReferencePoint,
    pub check: ReferenceCheckView,
}
#[derive(Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ReferenceCheckView {
    Checked {
        distance_along_meters: f64,
        miss_meters: f64,
        angle_degrees: f64,
        closest_point_meters: [f64; 3],
    },
    Unavailable {
        reason: String,
    },
}
pub(super) fn check(
    root: &Value,
    fixture: &Value,
    profile: &Value,
    point: &ReferencePoint,
) -> Result<ReferenceCheckView, String> {
    let model = super::super::model(profile)?.ok_or("未定义两轴物理模型")?;
    let placement = array(&root["stage"], "placements")
        .iter()
        .find(|v| v["fixtureId"] == fixture["id"])
        .ok_or("尚未布置灯具安装位置")?;
    let placement: FixturePlacement =
        serde_json::from_value(placement.clone()).map_err(|_| "安装位置无效")?;
    let zero: Option<FixtureZero> = fixture
        .get("zeroCorrection")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "零偏无效"))
        .transpose()?;
    let result = model
        .head(zero.as_ref())?
        .check_reference(
            Installation {
                position_meters: placement.position_meters.numbers(100_000.0)?,
                rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0)?,
            },
            JointAngles {
                pan_degrees: model
                    .pan
                    .decode(point.pan_value, super::super::fine(profile, "pan"))?,
                tilt_degrees: model
                    .tilt
                    .decode(point.tilt_value, super::super::fine(profile, "tilt"))?,
            },
            point.target_meters.numbers(100_000.0)?,
        )
        .map_err(|e| match e {
            stagemaster_spatial::positioning::Error::TargetAtPivot => {
                "目标与灯具轴心重合，无法检查方向"
            }
            _ => "参考点、灯位或运动模型无效",
        })?;
    Ok(ReferenceCheckView::Checked {
        distance_along_meters: result.distance_along_meters,
        miss_meters: result.miss_meters,
        angle_degrees: result.angle_degrees,
        closest_point_meters: result.closest_point_meters,
    })
}
pub(crate) fn project(
    root: &Value,
    fixture: &Value,
    profile: &Value,
) -> Option<PositionReferenceView> {
    let record: PositionReference = read(fixture).expect("validated references")?;
    let compatible = matches_profile(&record, profile);
    let points = record
        .points
        .into_iter()
        .map(|point| {
            let result = if compatible {
                check(root, fixture, profile, &point)
            } else {
                Err(format!(
                    "档案已改变；旧参考记录不能用于当前模式“{}”",
                    text(profile, "name")
                ))
            };
            ReferencePointView {
                point,
                check: result.unwrap_or_else(|reason| ReferenceCheckView::Unavailable { reason }),
            }
        })
        .collect();
    Some(PositionReferenceView {
        profile_id: record.profile_id,
        profile_revision: record.profile_revision,
        compatible,
        points,
    })
}
