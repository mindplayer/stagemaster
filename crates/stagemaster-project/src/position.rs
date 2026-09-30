//! Physical fixture positioning; UI and renderers never own inverse kinematics.
use crate::{Document, SpatialVector3, array, editing, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stagemaster_spatial::{
    Installation,
    positioning::{AxisRange, Branch, IntersectingHead, JointAngles, ZeroCorrection},
};
use std::collections::BTreeSet;

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
fn default(p: &Value, key: &str) -> u16 {
    u16::try_from(
        array(p, "attributes")
            .iter()
            .find(|a| a["key"] == key)
            .and_then(|a| a["default"]["value"].as_u64())
            .unwrap_or_default(),
    )
    .expect("validated default")
}
fn scene_value(root: &Value, scene: &Value, id: &str, key: &str, p: &Value) -> u16 {
    let entry = array(scene, "assignments")
        .iter()
        .find(|a| a["target"]["fixtureId"] == id && a["target"]["attribute"] == key);
    let Some(e) = entry.filter(|e| e["operation"] == "set") else {
        return default(p, key);
    };
    let v = if e["source"]["kind"] == "literal" {
        &e["source"]["value"]
    } else {
        let preset = array(&root["lighting"], "presets")
            .iter()
            .find(|p| p["id"] == e["source"]["presetId"])
            .expect("validated preset");
        &array(preset, "values")
            .iter()
            .find(|v| v["target"] == e["target"])
            .expect("validated target")["value"]
    };
    u16::try_from(v["value"].as_u64().expect("validated value")).expect("validated value")
}
fn solution_error(error: stagemaster_spatial::positioning::Error) -> &'static str {
    use stagemaster_spatial::positioning::Error;
    match error {
        Error::TargetAtPivot => "目标与灯具轴心重合",
        Error::Unreachable => "目标超出此灯具的机械行程",
        Error::InvalidTarget => "目标坐标无效",
        _ => "灯位、行程或零偏无效",
    }
}
fn resolve(
    root: &Value,
    command: &PositionEdit,
    scene: &Value,
    f: &Value,
    p: &Value,
    id: &str,
) -> Result<[Option<u16>; 2], String> {
    let m = model(p)?.ok_or("未定义两轴运动模型")?;
    let old = JointAngles {
        pan_degrees: m
            .pan
            .decode(scene_value(root, scene, id, "pan", p), fine(p, "pan"))?,
        tilt_degrees: m
            .tilt
            .decode(scene_value(root, scene, id, "tilt", p), fine(p, "tilt"))?,
    };
    let values = match command {
        PositionEdit::Home { .. } => [Some(default(p, "pan")), Some(default(p, "tilt"))],
        PositionEdit::Axes {
            pan_degrees,
            tilt_degrees,
            ..
        } => {
            if pan_degrees.is_none() && tilt_degrees.is_none() {
                return Err("至少填写一个轴的角度".into());
            }
            [
                pan_degrees
                    .as_ref()
                    .map(|v| m.pan.encode(decimal(v, 3600.0)?, fine(p, "pan")))
                    .transpose()?,
                tilt_degrees
                    .as_ref()
                    .map(|v| m.tilt.encode(decimal(v, 3600.0)?, fine(p, "tilt")))
                    .transpose()?,
            ]
        }
        PositionEdit::Aim {
            target_meters,
            branch,
            ..
        } => {
            let placement = array(&root["stage"], "placements")
                .iter()
                .find(|v| v["fixtureId"] == id)
                .ok_or("尚未布置灯位，请先在舞台设置安装位置")?;
            let placement: crate::FixturePlacement =
                serde_json::from_value(placement.clone()).map_err(|_| "灯位无效")?;
            let install = Installation {
                position_meters: placement.position_meters.numbers(100_000.0)?,
                rotation_degrees_xyz: placement.rotation_degrees_xyz.numbers(3600.0)?,
            };
            let z: Option<FixtureZero> = f
                .get("zeroCorrection")
                .map(|v| serde_json::from_value(v.clone()).map_err(|_| "零偏无效"))
                .transpose()?;
            let branch = match branch.as_deref() {
                None => None,
                Some("front") => Some(Branch::Front),
                Some("back") => Some(Branch::Back),
                _ => return Err("指向分支无效".into()),
            };
            let solved = m
                .head(z.as_ref())?
                .solve(install, target_meters.numbers(100_000.0)?, old, branch)
                .map_err(solution_error)?;
            [
                Some(m.pan.encode(solved.angles.pan_degrees, fine(p, "pan"))?),
                Some(m.tilt.encode(solved.angles.tilt_degrees, fine(p, "tilt"))?),
            ]
        }
        PositionEdit::Calibrate { .. } => unreachable!(),
    };
    Ok::<_, String>(values)
}
pub(super) fn apply(root: &mut Value, command: PositionEdit) -> Result<(), String> {
    if let PositionEdit::Calibrate {
        fixture_id,
        correction,
    } = command
    {
        let (f, p) = fixture_profile(root, &fixture_id)?;
        let m = model(p)?.ok_or("所选灯具未定义两轴模型")?;
        m.head(correction.as_ref())
            .map_err(|e| format!("{}：{e}", text(f, "name")))?;
        let f = editing::find(editing::list(root, "fixtures")?, &fixture_id)?;
        if let Some(z) = correction {
            f["zeroCorrection"] = json!(z);
        } else {
            f.as_object_mut().expect("fixture").remove("zeroCorrection");
        }
        require(root);
        return Ok(());
    }
    let (scene_id, ids) = match &command {
        PositionEdit::Axes {
            scene_id,
            fixture_ids,
            ..
        }
        | PositionEdit::Aim {
            scene_id,
            fixture_ids,
            ..
        }
        | PositionEdit::Home {
            scene_id,
            fixture_ids,
        } => (scene_id, fixture_ids),
        PositionEdit::Calibrate { .. } => unreachable!(),
    };
    if ids.is_empty() || ids.len() > 128 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("请选择 1–128 台不重复的摇头灯".into());
    }
    let scene = array(&root["lighting"], "scenes")
        .iter()
        .find(|s| s["id"] == *scene_id)
        .ok_or("场景不存在")?;
    let mut changes = Vec::new();
    for id in ids {
        let (f, p) = fixture_profile(root, id)?;
        let result = resolve(root, &command, scene, f, p, id)
            .map_err(|e| format!("灯具“{}”：{e}", text(f, "name")))?;
        for (key, value) in ["pan", "tilt"].into_iter().zip(result) {
            if let Some(value) = value {
                changes.push(editing::EditCommand::SetSceneValue {
                    scene_id: scene_id.clone(),
                    fixture_id: id.clone(),
                    attribute: key.into(),
                    mode: editing::ValueMode::Literal,
                    value,
                });
            }
        }
    }
    for change in changes {
        editing::apply(root, change)?;
    }
    Ok(())
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
