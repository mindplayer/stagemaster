use super::{decimal, fine, scene_value, solution_error};
use crate::{FixtureZero, PositionAxis, PositionModel, array, text};
use serde_json::Value;
use stagemaster_spatial::positioning::JointAngles;

fn check_effect(scene: &Value, fixture: &Value, axes: &[&str]) -> Result<(), String> {
    for effect in array(scene, "effects") {
        if effect["enabled"] != true || !array(effect, "fixtureIds").contains(&fixture["id"]) {
            continue;
        }
        if effect["waveform"] == "worldLine"
            || array(effect, "channels")
                .iter()
                .any(|channel| axes.iter().any(|key| channel["attribute"] == *key))
        {
            return Err(format!(
                "位置正由效果“{}”控制，请先停用该效果",
                text(effect, "name")
            ));
        }
    }
    Ok(())
}

pub(super) fn offset(
    root: &Value,
    scene: &Value,
    f: &Value,
    p: &Value,
    model: &PositionModel,
    old: JointAngles,
    deltas: [&Option<String>; 2],
) -> Result<[Option<u16>; 2], String> {
    let parse = |v: &Option<String>| v.as_deref().map(|s| decimal(s, 3600.0)).transpose();
    let deltas = [parse(deltas[0])?, parse(deltas[1])?];
    if deltas.iter().all(|v| v.unwrap_or(0.0) == 0.0) {
        return Err("至少填写一个非零轴增量".into());
    }
    let axes: Vec<_> = ["pan", "tilt"]
        .into_iter()
        .zip(deltas)
        .filter_map(|(key, delta)| (delta.unwrap_or(0.0) != 0.0).then_some(key))
        .collect();
    check_effect(scene, f, &axes)?;
    let apply = |key: &str, axis: &PositionAxis, degrees: f64, delta: Option<f64>| {
        let Some(delta) = delta.filter(|v| *v != 0.0) else {
            return Ok(None);
        };
        let encoded = axis.encode(degrees + delta, fine(p, key))?;
        let before = scene_value(root, scene, text(f, "id"), key, p);
        let same = if fine(p, key) {
            encoded == before
        } else {
            encoded >> 8 == before >> 8
        };
        if same {
            return Err(format!(
                "{}增量小于通道可分辨精度，请增大调整量",
                if key == "pan" { "水平" } else { "垂直" }
            ));
        }
        Ok(Some(encoded))
    };
    Ok([
        apply("pan", &model.pan, old.pan_degrees, deltas[0])?,
        apply("tilt", &model.tilt, old.tilt_degrees, deltas[1])?,
    ])
}

pub(super) fn flip(
    scene: &Value,
    f: &Value,
    p: &Value,
    model: &PositionModel,
    old: JointAngles,
) -> Result<[Option<u16>; 2], String> {
    check_effect(scene, f, &["pan", "tilt"])?;
    let zero: Option<FixtureZero> = f
        .get("zeroCorrection")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "零偏无效"))
        .transpose()?;
    let solution = model
        .head(zero.as_ref())?
        .flip(old)
        .map_err(solution_error)?;
    Ok([
        Some(
            model
                .pan
                .encode(solution.angles.pan_degrees, fine(p, "pan"))?,
        ),
        Some(
            model
                .tilt
                .encode(solution.angles.tilt_degrees, fine(p, "tilt"))?,
        ),
    ])
}
