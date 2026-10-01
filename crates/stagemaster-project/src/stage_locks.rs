//! Editing guards, independent of light playback and UI interaction permissions.
use crate::{array, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub(super) const CAPABILITY: &str = "stage.edit-locks";
const LIMIT: usize = 1600;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StageLockKind {
    Space,
    Construction,
    Placement,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageEditLock {
    pub kind: StageLockKind,
    pub target_id: String,
}
fn read(root: &Value) -> Result<Vec<StageEditLock>, String> {
    serde_json::from_value(
        root["stage"]
            .get("editLocks")
            .cloned()
            .unwrap_or_else(|| json!([])),
    )
    .map_err(|_| "场地锁定数据无效".into())
}
fn objects(root: &Value) -> BTreeMap<(StageLockKind, &str), &Value> {
    [
        ("spaces", StageLockKind::Space, "id"),
        ("constructions", StageLockKind::Construction, "id"),
        ("placements", StageLockKind::Placement, "fixtureId"),
    ]
    .into_iter()
    .flat_map(|(key, kind, field)| {
        array(&root["stage"], key)
            .iter()
            .map(move |v| ((kind, text(v, field)), v))
    })
    .collect()
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let locks = read(root)?;
    if !locks.is_empty()
        && !array(root, "requires")
            .iter()
            .any(|v| v["key"] == CAPABILITY && v["version"] == 1)
    {
        return Err("工程缺少场地锁定能力声明".into());
    }
    check_targets(&locks, &objects(root), false)
}
fn check_targets(
    targets: &[StageEditLock],
    objects: &BTreeMap<(StageLockKind, &str), &Value>,
    nonempty: bool,
) -> Result<(), String> {
    if targets.len() > LIMIT || (nonempty && targets.is_empty()) {
        return Err("一次锁定操作需要 1–1600 个对象".into());
    }
    let mut unique = BTreeSet::new();
    for target in targets {
        if !unique.insert(target) {
            return Err("场地锁定对象重复".into());
        }
        if !objects.contains_key(&(target.kind, target.target_id.as_str())) {
            return Err("场地锁定对象不存在".into());
        }
    }
    Ok(())
}
pub(super) fn set(
    root: &mut Value,
    targets: Vec<StageEditLock>,
    locked: bool,
) -> Result<(), String> {
    check_targets(&targets, &objects(root), true)?;
    let mut locks = read(root)?.into_iter().collect::<BTreeSet<_>>();
    for target in targets {
        if locked {
            locks.insert(target);
        } else {
            locks.remove(&target);
        }
    }
    if locks.is_empty() {
        root["stage"]
            .as_object_mut()
            .ok_or("场地数据缺失")?
            .remove("editLocks");
    } else {
        root["stage"]["editLocks"] = json!(locks);
    }
    let requires = root["requires"].as_array_mut().ok_or("工程能力声明缺失")?;
    if locks.is_empty() {
        requires.retain(|v| v["key"] != CAPABILITY);
    } else if !requires.iter().any(|v| v["key"] == CAPABILITY) {
        requires.push(json!({"key": CAPABILITY,"version":1}));
    }
    Ok(())
}
struct Protected {
    target: StageEditLock,
    label: String,
    value: Value,
}
pub(super) struct Snapshot(Vec<Protected>);
fn projection(root: &Value, target: &StageEditLock, value: &Value) -> Value {
    match target.kind {
        StageLockKind::Placement => json!([
            value,
            array(&root["stage"], "attachments")
                .iter()
                .find(|a| a["fixtureId"] == target.target_id)
        ]),
        StageLockKind::Construction if value["shape"]["kind"] == "enclosure" => {
            let space = array(&root["stage"], "spaces")
                .iter()
                .find(|s| s["id"] == value["shape"]["spaceId"]);
            json!([
                value,
                space.map(|s| json!([
                    s["outlineMeters"],
                    s["floorElevationMeters"],
                    s["clearHeightMeters"]
                ]))
            ])
        }
        _ => value.clone(),
    }
}
pub(super) fn capture(root: &Value) -> Result<Snapshot, String> {
    let locks = read(root)?;
    if locks.is_empty() {
        return Ok(Snapshot(Vec::new()));
    }
    let objects = objects(root);
    let mut protected = Vec::with_capacity(locks.len());
    for target in locks {
        let value = objects
            .get(&(target.kind, target.target_id.as_str()))
            .ok_or("场地锁定对象不存在")?;
        let label = if target.kind == StageLockKind::Placement {
            array(&root["lighting"], "fixtures")
                .iter()
                .find(|f| f["id"] == target.target_id)
                .map_or("灯位", |f| text(f, "name"))
        } else {
            text(value, "name")
        };
        let value = projection(root, &target, value);
        protected.push(Protected {
            target,
            label: label.into(),
            value,
        });
    }
    Ok(Snapshot(protected))
}
impl Snapshot {
    pub(super) fn verify(&self, root: &Value) -> Result<(), String> {
        if self.0.is_empty() {
            return Ok(());
        }
        let objects = objects(root);
        for protected in &self.0 {
            let target = &protected.target;
            let next = objects
                .get(&(target.kind, target.target_id.as_str()))
                .map(|v| projection(root, target, v));
            if next.as_ref() != Some(&protected.value) {
                return Err(format!(
                    "场地对象“{}”已锁定，请先解锁；本次操作未应用",
                    protected.label
                ));
            }
        }
        Ok(())
    }
}
