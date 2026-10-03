//! Atomic world-space translations. Fixture relationships and installation angles are untouched.
use crate::{SpatialVector3, array};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn apply(
    root: &mut Value,
    ids: &[String],
    delta: &SpatialVector3,
) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 256 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("一次移动需要 1–256 台不重复的已布置灯具".into());
    }
    let offsets = offsets(delta)?;
    let mut changes = Vec::with_capacity(ids.len());
    for id in ids {
        let (index, placement) = array(&root["stage"], "placements")
            .iter()
            .enumerate()
            .find(|(_, p)| p["fixtureId"] == *id)
            .ok_or("所选灯具已不存在或尚未布置")?;
        let mut position = placement["positionMeters"].clone();
        for (key, offset) in ["x", "y", "z"].into_iter().zip(offsets) {
            shift(&mut position[key], offset)?;
        }
        changes.push((index, position));
    }
    let placements = crate::stage::list(root, "placements")?;
    for (index, position) in changes {
        placements[index]["positionMeters"] = position;
    }
    Ok(())
}

pub(super) fn offsets(delta: &SpatialVector3) -> Result<[f64; 3], String> {
    for field in [&delta.x, &delta.y, &delta.z] {
        let unsigned = field.strip_prefix('-').unwrap_or(field);
        let parts = unsigned.split('.').collect::<Vec<_>>();
        if field.len() > 32
            || parts.len() > 2
            || parts
                .iter()
                .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
            || parts.get(1).is_some_and(|part| part.len() > 6)
        {
            return Err("位移需要最多六位小数的十进制数".into());
        }
    }
    delta.numbers(200_000.0)
}

pub(super) fn shift(value: &mut Value, offset: f64) -> Result<(), String> {
    if offset == 0.0 {
        return Ok(());
    }
    let previous =
        crate::stage::decimal(value.as_str().ok_or("场地坐标无效")?, -100_000.0, 100_000.0)?;
    let next = previous + offset;
    if !(-100_000.0..=100_000.0).contains(&next) {
        return Err("整组移动后有对象超出场地范围，未修改任何对象".into());
    }
    let formatted = format!("{next:.6}");
    let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
    *value = if formatted == "-0" { "0" } else { formatted }.into();
    Ok(())
}
