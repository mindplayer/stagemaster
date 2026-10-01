use super::{FixtureZero, PositionEdit, decimal, fine, fixture_profile, model, require};
use crate::{array, editing, text};
use serde_json::{Value, json};
use stagemaster_spatial::{
    Installation,
    positioning::{Branch, JointAngles},
};
use std::collections::BTreeSet;
mod adjustment;

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
        Error::SingularFlip => "当前方向位于转轴奇点，无法确定另一支架姿态",
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
        PositionEdit::OffsetAxes {
            pan_degrees,
            tilt_degrees,
            ..
        } => adjustment::offset(root, scene, f, p, &m, old, [pan_degrees, tilt_degrees])?,
        PositionEdit::Flip { .. } => adjustment::flip(scene, f, p, &m, old)?,
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
pub(crate) fn apply(root: &mut Value, command: PositionEdit) -> Result<(), String> {
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
        | PositionEdit::OffsetAxes {
            scene_id,
            fixture_ids,
            ..
        }
        | PositionEdit::Flip {
            scene_id,
            fixture_ids,
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
