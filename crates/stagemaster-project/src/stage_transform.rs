//! Group transforms in project world coordinates; renderer math is only a disposable preview.
use crate::{array, text};
use serde_json::Value;
use std::collections::BTreeSet;

fn scalar(value: &str, min: f64, max: f64) -> Result<f64, String> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let parts = unsigned.split('.').collect::<Vec<_>>();
    if value.len() > 32
        || parts.len() > 2
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
        || parts.get(1).is_some_and(|p| p.len() > 6)
    {
        return Err("变换参数需要最多六位小数的十进制数".into());
    }
    crate::stage::decimal(value, min, max)
}
fn decimal(value: f64) -> String {
    let value = format!("{value:.6}");
    let value = value.trim_end_matches('0').trim_end_matches('.');
    if value == "-0" {
        "0".into()
    } else {
        value.into()
    }
}

pub(super) fn apply(
    root: &mut Value,
    ids: &[String],
    yaw: &str,
    scale: &str,
) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 256 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("一次变换需要 1–256 台不重复的已布置灯具".into());
    }
    let yaw = scalar(yaw, -360.0, 360.0)? % 360.0;
    let scale = scalar(scale, 0.01, 100.0)?;
    let mut originals = Vec::with_capacity(ids.len());
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    for id in ids {
        let (index, placement) = array(&root["stage"], "placements")
            .iter()
            .enumerate()
            .find(|(_, p)| p["fixtureId"] == *id)
            .ok_or("所选灯具已不存在或尚未布置")?;
        let mut position = [0.0; 3];
        for (axis, key) in ["x", "y", "z"].into_iter().enumerate() {
            position[axis] = crate::stage::decimal(
                text(&placement["positionMeters"], key),
                -100_000.0,
                100_000.0,
            )?;
            low[axis] = low[axis].min(position[axis]);
            high[axis] = high[axis].max(position[axis]);
        }
        originals.push((index, placement.clone(), position));
    }
    if yaw == 0.0 && (scale - 1.0).abs() < f64::EPSILON {
        return Ok(());
    }
    let center = std::array::from_fn::<_, 3, _>(|axis| (low[axis] + high[axis]) * 0.5);
    let (sin, cos) = yaw.to_radians().sin_cos();
    let mut changes = Vec::with_capacity(ids.len());
    for (index, mut placement, position) in originals {
        let local = std::array::from_fn::<_, 3, _>(|axis| (position[axis] - center[axis]) * scale);
        let next = [
            center[0] + local[0] * cos - local[1] * sin,
            center[1] + local[0] * sin + local[1] * cos,
            center[2] + local[2],
        ];
        for (axis, key) in ["x", "y", "z"].into_iter().enumerate() {
            if !(-100_000.0..=100_000.0).contains(&next[axis]) {
                return Err("整组变换后有灯位超出场地范围，未修改任何灯具".into());
            }
            // Preserve exact unchanged coordinates, including no-op single-fixture spacing.
            if (next[axis] - position[axis]).abs() >= 0.000_000_5 {
                placement["positionMeters"][key] = decimal(next[axis]).into();
            }
        }
        if yaw != 0.0 {
            let previous = crate::stage::decimal(
                text(&placement["rotationDegreesXYZ"], "z"),
                -3600.0,
                3600.0,
            )?;
            placement["rotationDegreesXYZ"]["z"] =
                decimal((previous + yaw + 180.0).rem_euclid(360.0) - 180.0).into();
        }
        changes.push((index, placement));
    }
    let placements = crate::stage::list(root, "placements")?;
    for (index, placement) in changes {
        placements[index] = placement;
    }
    Ok(())
}
