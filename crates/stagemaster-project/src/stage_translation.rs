//! Atomic world-space translations. Fixture relationships and installation angles are untouched.
use crate::{SpatialVector3, array, text};
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
    let offsets = delta.numbers(200_000.0)?;
    let mut changes = Vec::with_capacity(ids.len());
    for id in ids {
        let (index, placement) = array(&root["stage"], "placements")
            .iter()
            .enumerate()
            .find(|(_, p)| p["fixtureId"] == *id)
            .ok_or("所选灯具已不存在或尚未布置")?;
        let mut position = placement["positionMeters"].clone();
        for (key, offset) in ["x", "y", "z"].into_iter().zip(offsets) {
            if offset == 0.0 {
                continue;
            }
            let previous = crate::stage::decimal(text(&position, key), -100_000.0, 100_000.0)?;
            let next = previous + offset;
            if !(-100_000.0..=100_000.0).contains(&next) {
                return Err("整组移动后有灯位超出场地范围，未修改任何灯具".into());
            }
            let value = format!("{next:.6}");
            let value = value.trim_end_matches('0').trim_end_matches('.');
            position[key] = if value == "-0" { "0" } else { value }.into();
        }
        changes.push((index, position));
    }
    let placements = crate::stage::list(root, "placements")?;
    for (index, position) in changes {
        placements[index]["positionMeters"] = position;
    }
    Ok(())
}
