//! Per-fixture setpoint records, never execution data or measured device feedback.
use crate::{SpatialVector3, array, text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
pub(super) mod edit;
pub(crate) mod view;
pub(crate) const CAPABILITY: &str = "lighting.position-reference";
pub const MAX_POINTS_PER_FIXTURE: usize = 16;
pub const MAX_POINTS_PER_PROJECT: usize = 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionReference {
    pub profile_id: String,
    pub profile_revision: String,
    pub points: Vec<ReferencePoint>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferencePoint {
    pub id: String,
    pub name: String,
    pub target_meters: SpatialVector3,
    /// Canonical normalized representation of the emitted coarse/fine precision.
    pub pan_value: u16,
    pub tilt_value: u16,
    pub source: ReferenceSource,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceSource {
    SceneSetpoint,
}

pub(super) fn read(fixture: &Value) -> Result<Option<PositionReference>, String> {
    fixture
        .get("positionReference")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "参考点记录字段无效".into()))
        .transpose()
}
pub(super) fn matches_profile(record: &PositionReference, profile: &Value) -> bool {
    record.profile_id == text(profile, "id") && record.profile_revision == text(profile, "revision")
}
pub(crate) fn validate(root: &Value) -> Result<(), String> {
    let mut total = 0;
    for fixture in array(&root["lighting"], "fixtures") {
        let Some(record) = read(fixture)? else {
            continue;
        };
        if !array(root, "requires")
            .iter()
            .any(|c| c["key"] == CAPABILITY && c["version"] == 1)
        {
            return Err("参考点记录缺少位置参考能力声明".into());
        }
        if record.points.is_empty() || record.points.len() > MAX_POINTS_PER_FIXTURE {
            return Err("每台灯具需要 1–16 个参考点".into());
        }
        total += record.points.len();
        if total > MAX_POINTS_PER_PROJECT {
            return Err("工程参考点总数不能超过 1024".into());
        }
        let mut names = BTreeSet::new();
        for point in &record.points {
            if point.name.trim().is_empty()
                || point.name.chars().count() > 256
                || !names.insert(point.name.trim())
            {
                return Err(format!(
                    "灯具“{}”的参考点名称为空、重复或过长",
                    text(fixture, "name")
                ));
            }
            point.target_meters.numbers(100_000.0)?;
        }
    }
    Ok(())
}
