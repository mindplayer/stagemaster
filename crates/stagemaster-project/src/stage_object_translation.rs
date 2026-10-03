//! Mixed world-space translation. Expand attachments once, then move every member once.
use crate::stage_translation::{offsets, shift};
use crate::{SpatialVector3, StageEditLock, StageLockKind, array, text};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn apply(
    root: &mut Value,
    targets: &[StageEditLock],
    delta: &SpatialVector3,
) -> Result<(), String> {
    if targets.is_empty()
        || targets.len() > 256
        || targets.iter().collect::<BTreeSet<_>>().len() != targets.len()
    {
        return Err("一次移动需要 1–256 个不重复的场地对象".into());
    }
    let offsets = offsets(delta)?;
    let mut constructions = BTreeSet::new();
    let mut placements = BTreeSet::new();
    for target in targets {
        match target.kind {
            StageLockKind::Space => return Err("空间和围护不支持混合移动，请单独编辑空间".into()),
            StageLockKind::Placement => {
                if !array(&root["stage"], "placements")
                    .iter()
                    .any(|p| p["fixtureId"] == target.target_id)
                {
                    return Err("所选灯具已不存在或尚未布置".into());
                }
                placements.insert(target.target_id.as_str());
            }
            StageLockKind::Construction => {
                let construction = array(&root["stage"], "constructions")
                    .iter()
                    .find(|c| c["id"] == target.target_id)
                    .ok_or("所选构件已不存在")?;
                if !matches!(
                    text(&construction["shape"], "kind"),
                    "rig" | "seating" | "platform"
                ) {
                    return Err("空间和围护不支持混合移动，请单独编辑空间".into());
                }
                constructions.insert(target.target_id.as_str());
            }
        }
    }
    // Owned IDs release the immutable root borrow before applying the candidate transaction.
    let mut affected = placements
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for attachment in array(&root["stage"], "attachments") {
        if constructions.contains(text(attachment, "constructionId")) {
            affected.insert(text(attachment, "fixtureId").to_owned());
        }
    }
    for construction in crate::stage::list(root, "constructions")? {
        if !constructions.contains(text(construction, "id")) {
            continue;
        }
        let shape = &mut construction["shape"];
        if shape["kind"] == "platform" {
            for point in shape["outlineMeters"]
                .as_array_mut()
                .ok_or("地台轮廓无效")?
            {
                shift(&mut point[0], offsets[0])?;
                shift(&mut point[1], offsets[1])?;
            }
            shift(&mut shape["baseElevationMeters"], offsets[2])?;
        } else {
            position(&mut shape["positionMeters"], offsets)?;
        }
    }
    for placement in crate::stage::list(root, "placements")? {
        if affected.contains(text(placement, "fixtureId")) {
            position(&mut placement["positionMeters"], offsets)?;
        }
    }
    Ok(())
}
fn position(value: &mut Value, offsets: [f64; 3]) -> Result<(), String> {
    for (axis, offset) in ["x", "y", "z"].into_iter().zip(offsets) {
        shift(&mut value[axis], offset)?;
    }
    Ok(())
}
