//! Atomic structural edits on stable step identities, independent of the running plan.
use crate::{id, sequence, text};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StepGroupOperation {
    Copy { before_id: Option<String> },
    Move { before_id: Option<String> },
    Remove {},
    Timing { patch: crate::StepTimingPatch },
}

pub(super) fn apply(
    root: &mut Value,
    sequence_id: &str,
    ids: &[String],
    operation: StepGroupOperation,
) -> Result<(), String> {
    if ids.is_empty() || ids.len() > stagemaster_playback::MAX_STEPS {
        return Err("成组操作请选择 1—1024 个步骤".into());
    }
    let selected: BTreeSet<_> = ids.iter().map(String::as_str).collect();
    if selected.len() != ids.len() {
        return Err("所选步骤身份重复".into());
    }
    let steps = sequence::steps_mut(root, sequence_id)?;
    if !selected
        .iter()
        .all(|id| steps.iter().any(|s| text(s, "id") == *id))
    {
        return Err("部分所选步骤已不存在，请重新选择".into());
    }
    match operation {
        StepGroupOperation::Timing { patch } => {
            crate::sequence_timing::apply(steps, &selected, &patch)?;
        }
        StepGroupOperation::Remove {} => {
            if selected.len() == steps.len() {
                return Err("列表至少保留一个步骤；请取消一项选择，或删除整个列表".into());
            }
            steps.retain(|s| !selected.contains(text(s, "id")));
        }
        StepGroupOperation::Copy { before_id } => {
            if steps.len() + selected.len() > stagemaster_playback::MAX_STEPS {
                return Err("复制后超过 1024 步，请拆分列表".into());
            }
            let at = destination(steps, before_id.as_deref())?;
            let copies: Vec<_> = steps
                .iter()
                .filter(|s| selected.contains(text(s, "id")))
                .cloned()
                .collect();
            for (offset, mut copy) in copies.into_iter().enumerate() {
                copy["id"] = id().into();
                copy["number"] = sequence::next_number(steps)?.into();
                steps.insert(at + offset, copy);
            }
        }
        StepGroupOperation::Move { before_id } => {
            if before_id.as_deref().is_some_and(|id| selected.contains(id)) {
                return Err("移动目标不能是所选步骤，请选择组外步骤或列表末尾".into());
            }
            destination(steps, before_id.as_deref())?;
            let moved: Vec<_> = steps
                .iter()
                .filter(|s| selected.contains(text(s, "id")))
                .cloned()
                .collect();
            steps.retain(|s| !selected.contains(text(s, "id")));
            let at = destination(steps, before_id.as_deref())?;
            steps.splice(at..at, moved);
        }
    }
    Ok(())
}
fn destination(steps: &[Value], before: Option<&str>) -> Result<usize, String> {
    before.map_or(Ok(steps.len()), |id| {
        steps
            .iter()
            .position(|s| text(s, "id") == id)
            .ok_or_else(|| "目标步骤已不存在，请重新选择位置".into())
    })
}
