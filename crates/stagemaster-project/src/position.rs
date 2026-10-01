//! Physical fixture positioning; UI and renderers never own inverse kinematics.
use crate::{Document, SpatialVector3, array};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stagemaster_spatial::positioning::{AxisRange, IntersectingHead, ZeroCorrection};
mod edit;
pub(super) mod reference;
pub(super) use edit::apply;
pub use reference::view::{PositionReferenceView, ReferenceCheckView, ReferencePointView};
pub use reference::{PositionReference, ReferencePoint, ReferenceSource};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionAxis {
    pub min_degrees: String,
    pub max_degrees: String,
    pub reversed: bool,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionModel {
    pub kind: String,
    pub pan: PositionAxis,
    pub tilt: PositionAxis,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureZero {
    pub pan_degrees: String,
    pub tilt_degrees: String,
}
#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PositionEdit {
    Axes {
        scene_id: String,
        fixture_ids: Vec<String>,
        pan_degrees: Option<String>,
        tilt_degrees: Option<String>,
    },
    OffsetAxes {
        scene_id: String,
        fixture_ids: Vec<String>,
        pan_degrees: Option<String>,
        tilt_degrees: Option<String>,
    },
    Flip {
        scene_id: String,
        fixture_ids: Vec<String>,
    },
    Aim {
        scene_id: String,
        fixture_ids: Vec<String>,
        target_meters: SpatialVector3,
        branch: Option<String>,
    },
    Home {
        scene_id: String,
        fixture_ids: Vec<String>,
    },
    Calibrate {
        fixture_id: String,
        correction: Option<FixtureZero>,
    },
    CaptureReference {
        scene_id: String,
        fixture_id: String,
        name: String,
        target_meters: SpatialVector3,
    },
    RemoveReference {
        fixture_id: String,
        point_id: String,
    },
    ClearReferences {
        fixture_id: String,
    },
}
fn decimal(s: &str, limit: f64) -> Result<f64, String> {
    crate::stage::decimal(s, -limit, limit)
}
impl PositionAxis {
    /// # Errors
    /// Rejects unbounded or reversed physical ranges; output reversal is independent.
    pub fn range(&self) -> Result<AxisRange, String> {
        let range = AxisRange {
            min_degrees: decimal(&self.min_degrees, 3600.0)?,
            max_degrees: decimal(&self.max_degrees, 3600.0)?,
        };
        if range.min_degrees >= range.max_degrees {
            return Err("轴最小角度必须小于最大角度".into());
        }
        Ok(range)
    }
    /// Decode the actual integer resolution emitted by the DMX encoder.
    /// # Errors
    /// Rejects invalid physical definitions.
    pub fn decode(&self, value: u16, fine: bool) -> Result<f64, String> {
        let range = self.range()?;
        let t = if fine {
            f64::from(value) / 65535.0
        } else {
            f64::from(value >> 8) / 255.0
        };
        let t = if self.reversed { 1.0 - t } else { t };
        Ok(
            (range.min_degrees + (range.max_degrees - range.min_degrees) * t)
                .clamp(range.min_degrees, range.max_degrees),
        )
    }
    /// # Errors
    /// Rejects an unreachable angle instead of clamping it into travel.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn encode(&self, degrees: f64, fine: bool) -> Result<u16, String> {
        let r = self.range()?;
        if !degrees.is_finite() || degrees < r.min_degrees || degrees > r.max_degrees {
            return Err(format!(
                "角度须在 {}–{}° 之间",
                self.min_degrees, self.max_degrees
            ));
        }
        let t = (degrees - r.min_degrees) / (r.max_degrees - r.min_degrees);
        let t = if self.reversed { 1.0 - t } else { t };
        Ok(if fine {
            (t * 65535.0).round() as u16
        } else {
            (t * 255.0).round() as u16 * 257
        })
    }
}
impl PositionModel {
    /// # Errors
    /// Only the declared intersecting orthogonal model is implemented.
    pub fn head(&self, zero: Option<&FixtureZero>) -> Result<IntersectingHead, String> {
        if self.kind != "intersectingOrthogonal" {
            return Err("尚不支持此运动模型".into());
        }
        let correction = zero
            .map(|z| {
                Ok::<_, String>(ZeroCorrection {
                    pan_degrees: decimal(&z.pan_degrees, 360.0)?,
                    tilt_degrees: decimal(&z.tilt_degrees, 360.0)?,
                })
            })
            .transpose()?
            .unwrap_or_default();
        let head = IntersectingHead {
            pan: self.pan.range()?,
            tilt: self.tilt.range()?,
            zero_correction: correction,
        };
        head.validate().map_err(|_| "两轴物理范围或零偏无效")?;
        Ok(head)
    }
}
pub(super) fn model(profile: &Value) -> Result<Option<PositionModel>, String> {
    profile
        .get("positioning")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "运动模型字段无效".into()))
        .transpose()
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let profiles = array(&root["lighting"], "profiles");
    let mut required = false;
    for p in profiles {
        if let Some(m) = model(p)? {
            required = true;
            m.head(None)?;
            for key in ["pan", "tilt"] {
                if !array(p, "attributes").iter().any(|a| {
                    a["key"] == key && a["mix"] == "ltp" && a["valueType"]["kind"] == "normalized"
                }) {
                    return Err("两轴模型需要水平与垂直归一化属性，混合方式为 LTP".into());
                }
            }
        }
    }
    for f in array(&root["lighting"], "fixtures") {
        if let Some(z) = f.get("zeroCorrection") {
            required = true;
            let p = profiles
                .iter()
                .find(|p| p["id"] == f["profileId"])
                .ok_or("灯具档案不存在")?;
            let m = model(p)?.ok_or("固定灯具不能设置轴零偏")?;
            let z: FixtureZero =
                serde_json::from_value(z.clone()).map_err(|_| "单灯零偏字段无效")?;
            m.head(Some(&z))?;
        }
    }
    if required
        && !array(root, "requires")
            .iter()
            .any(|c| c["key"] == "lighting.positioning" && c["version"] == 1)
    {
        return Err("摇头灯工程缺少位置能力声明".into());
    }
    Ok(())
}
pub(super) fn require(root: &mut Value) {
    if !array(root, "requires")
        .iter()
        .any(|c| c["key"] == "lighting.positioning")
    {
        root["requires"]
            .as_array_mut()
            .expect("validated requirements")
            .push(json!({"key":"lighting.positioning","version":1}));
    }
}
pub(super) fn fixture_profile<'a>(
    root: &'a Value,
    id: &str,
) -> Result<(&'a Value, &'a Value), String> {
    let f = array(&root["lighting"], "fixtures")
        .iter()
        .find(|f| f["id"] == id)
        .ok_or("灯具不存在")?;
    let p = array(&root["lighting"], "profiles")
        .iter()
        .find(|p| p["id"] == f["profileId"])
        .ok_or("档案不存在")?;
    Ok((f, p))
}
pub(super) fn fine(p: &Value, key: &str) -> bool {
    array(p, "channels")
        .iter()
        .any(|c| c["attribute"] == key && c["encoding"] == "u16-be")
}
impl Document {
    /// Read-only physical position projection for editors and preview consumers.
    /// # Errors
    /// Rejects missing profiles or malformed model data.
    pub fn position_model(&self, fixture_id: &str) -> Result<Option<PositionModel>, String> {
        let (_, p) = fixture_profile(&self.root, fixture_id)?;
        model(p)
    }
}
