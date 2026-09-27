//! Physical fixture associations. World placements remain the single coordinate authority.
use crate::stage::{ConstructionShape, FixturePlacement, SpatialVector3, StageView};
use crate::{array, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RigKind {
    Truss,
    Pipe,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigShape {
    pub rig_kind: RigKind,
    pub space_id: Option<String>,
    pub position_meters: SpatialVector3,
    pub yaw_degrees: String,
    pub length_meters: String,
    pub width_meters: String,
    pub height_meters: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigAttachment {
    pub fixture_id: String,
    pub construction_id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigLayout {
    pub start_margin_meters: String,
    pub end_margin_meters: String,
    pub drop_meters: String,
}
impl RigShape {
    pub(super) fn validate(&self) -> Result<(), String> {
        let p = self.position_meters.numbers(100_000.0)?;
        number(&self.yaw_degrees, -3600.0, 3600.0)?;
        let length = number(&self.length_meters, 0.1, 1000.0)?;
        let width = number(&self.width_meters, 0.02, 10.0)?;
        let height = number(&self.height_meters, 0.02, 10.0)?;
        let radius = length.hypot(width) / 2.0;
        if p[0].abs() + radius > 100_000.0
            || p[1].abs() + radius > 100_000.0
            || p[2].abs() + height / 2.0 > 100_000.0
        {
            return Err("支撑体边界超出场地范围".into());
        }
        Ok(())
    }
}
fn number(s: &str, min: f64, max: f64) -> Result<f64, String> {
    crate::stage::decimal(s, min, max)
}
fn coordinate(v: f64) -> String {
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}
pub(super) fn validate(stage: &StageView) -> Result<(), String> {
    let mut attached = BTreeSet::new();
    for a in &stage.attachments {
        if !attached.insert(&a.fixture_id) {
            return Err("灯具不能重复挂接支撑体".into());
        }
        let placement = stage
            .placements
            .iter()
            .find(|p| p.fixture_id == a.fixture_id)
            .ok_or("挂接灯具缺少灯位")?;
        let rig = stage
            .constructions
            .iter()
            .find(|r| r.id == a.construction_id)
            .and_then(|r| {
                if let ConstructionShape::Rig(r) = &r.shape {
                    Some(r)
                } else {
                    None
                }
            })
            .ok_or("挂接引用的桁架或灯杆不存在")?;
        if placement.space_id != rig.space_id {
            return Err("挂接灯具与支撑体必须属于同一空间；请先解除挂接再单独更换空间".into());
        }
    }
    Ok(())
}
pub(super) fn move_members(
    root: &mut Value,
    id: &str,
    shape: &ConstructionShape,
) -> Result<(), String> {
    let members = array(&root["stage"], "attachments")
        .iter()
        .filter(|a| a["constructionId"] == id)
        .map(|a| text(a, "fixtureId").to_owned())
        .collect::<BTreeSet<_>>();
    if members.is_empty() {
        return Ok(());
    }
    let ConstructionShape::Rig(new) = shape else {
        return Err("请先解除挂接再改变支撑体类型".into());
    };
    new.validate()?;
    let old: ConstructionShape = serde_json::from_value(
        array(&root["stage"], "constructions")
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("支撑体不存在")?["shape"]
            .clone(),
    )
    .map_err(|_| "支撑体格式无效")?;
    let ConstructionShape::Rig(old) = old else {
        return Err("挂接支撑体格式无效".into());
    };
    let from = old.position_meters.numbers(100_000.0)?;
    let to = new.position_meters.numbers(100_000.0)?;
    let degrees =
        number(&new.yaw_degrees, -3600.0, 3600.0)? - number(&old.yaw_degrees, -3600.0, 3600.0)?;
    let (sin, cos) = degrees.to_radians().sin_cos();
    for item in crate::stage::list(root, "placements")? {
        if !members.contains(text(item, "fixtureId")) {
            continue;
        }
        let mut p: FixturePlacement =
            serde_json::from_value(item.clone()).map_err(|_| "灯位格式无效")?;
        let v = p.position_meters.numbers(100_000.0)?;
        // Preserve authored decimals for an identity transform (e.g. rename/resize).
        if from.iter().zip(to).any(|(a, b)| (a - b).abs() > 0.0) || degrees != 0.0 {
            p.position_meters = SpatialVector3 {
                x: coordinate(to[0] + (v[0] - from[0]) * cos - (v[1] - from[1]) * sin),
                y: coordinate(to[1] + (v[0] - from[0]) * sin + (v[1] - from[1]) * cos),
                z: coordinate(to[2] + v[2] - from[2]),
            };
        }
        if degrees != 0.0 {
            p.rotation_degrees_xyz.z = coordinate(
                (number(&p.rotation_degrees_xyz.z, -3600.0, 3600.0)? + degrees + 180.0)
                    .rem_euclid(360.0)
                    - 180.0,
            );
        }
        p.space_id.clone_from(&new.space_id);
        *item = json!(p);
    }
    Ok(())
}
pub(super) fn detach(root: &mut Value, ids: &[String]) -> Result<(), String> {
    if root["stage"].get("attachments").is_some() {
        crate::stage::list(root, "attachments")?
            .retain(|a| !ids.iter().any(|id| a["fixtureId"] == *id));
    }
    Ok(())
}
pub(super) fn remove_rig(root: &mut Value, id: &str, detach_fixtures: bool) -> Result<(), String> {
    let members = array(&root["stage"], "attachments")
        .iter()
        .filter(|a| a["constructionId"] == id)
        .map(|a| text(a, "fixtureId").to_owned())
        .collect::<Vec<_>>();
    if !members.is_empty() && !detach_fixtures {
        return Err(format!(
            "支撑体仍挂接 {} 台灯具，请先解除挂接或选择保留灯位并删除",
            members.len()
        ));
    }
    detach(root, &members)
}
pub(super) fn attach(
    root: &mut Value,
    construction_id: Option<&str>,
    ids: &[String],
    layout: Option<&RigLayout>,
) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 256 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("挂灯需要 1–256 台不重复的灯具".into());
    }
    for id in ids {
        if !array(&root["lighting"], "fixtures")
            .iter()
            .any(|f| f["id"] == *id)
        {
            return Err("挂接的灯具不存在".into());
        }
    }
    let Some(id) = construction_id else {
        if layout.is_some() {
            return Err("解除挂接不能同时布灯".into());
        }
        return detach(root, ids);
    };
    let shape: ConstructionShape = serde_json::from_value(
        array(&root["stage"], "constructions")
            .iter()
            .find(|r| r["id"] == id)
            .ok_or("支撑体不存在")?["shape"]
            .clone(),
    )
    .map_err(|_| "支撑体格式无效")?;
    let ConstructionShape::Rig(rig) = shape else {
        return Err("只能挂接到桁架或灯杆".into());
    };
    let mut placements = Vec::with_capacity(ids.len());
    let origin = rig.position_meters.numbers(100_000.0)?;
    let (sin, cos) = number(&rig.yaw_degrees, -3600.0, 3600.0)?
        .to_radians()
        .sin_cos();
    let parameters = layout
        .map(|l| -> Result<_, String> {
            let length = number(&rig.length_meters, 0.1, 1000.0)?;
            let start = number(&l.start_margin_meters, 0.0, length)?;
            let end = number(&l.end_margin_meters, 0.0, length)?;
            if start + end > length || (ids.len() > 1 && start + end >= length) {
                return Err("两端余量过大，没有足够长度排列灯具".into());
            }
            Ok((
                -length / 2.0 + start,
                length - start - end,
                number(&l.drop_meters, 0.0, 1000.0)?,
            ))
        })
        .transpose()?;
    for (i, id) in ids.iter().enumerate() {
        let old = array(&root["stage"], "placements")
            .iter()
            .find(|p| p["fixtureId"] == *id);
        let mut p = old
            .map(|p| {
                serde_json::from_value::<FixturePlacement>(p.clone()).map_err(|_| "灯位格式无效")
            })
            .transpose()?
            .unwrap_or(FixturePlacement {
                fixture_id: id.clone(),
                space_id: rig.space_id.clone(),
                position_meters: SpatialVector3 {
                    x: "0".into(),
                    y: "0".into(),
                    z: "0".into(),
                },
                rotation_degrees_xyz: SpatialVector3 {
                    x: "0".into(),
                    y: "0".into(),
                    z: "0".into(),
                },
            });
        if let Some((start, span, drop)) = parameters {
            #[allow(clippy::cast_precision_loss)]
            let t = if ids.len() == 1 {
                0.5
            } else {
                i as f64 / (ids.len() - 1) as f64
            };
            let x = start + t * span;
            p.position_meters = SpatialVector3 {
                x: coordinate(origin[0] + x * cos),
                y: coordinate(origin[1] + x * sin),
                z: coordinate(origin[2] - number(&rig.height_meters, 0.02, 10.0)? / 2.0 - drop),
            };
        } else if old.is_none() {
            return Err("保持位置挂接需要已有灯位，请选择沿支撑体均布".into());
        }
        p.space_id.clone_from(&rig.space_id);
        placements.push(p);
    }
    detach(root, ids)?;
    if root["stage"].get("attachments").is_none() {
        root["stage"]["attachments"] = json!([]);
    }
    for p in placements {
        crate::stage::list(root, "attachments")?
            .push(json!({"fixtureId":p.fixture_id,"constructionId":id}));
        crate::stage::apply(root, crate::stage::StageEdit::PutPlacement { placement: p })?;
    }
    Ok(())
}
