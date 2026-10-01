//! Sparse timing patches share the existing per-step playback semantics.
use crate::{sequence::duration, text};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StepTimingPatch {
    pub delay_ms: Option<u64>,
    pub fade_ms: Option<u64>,
    pub advance: Option<StepAdvance>,
}
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StepAdvance {
    Manual {},
    After { wait_ms: u64 },
}
pub(super) fn apply(
    steps: &mut [Value],
    selected: &BTreeSet<&str>,
    patch: &StepTimingPatch,
) -> Result<(), String> {
    if patch.delay_ms.is_none() && patch.fade_ms.is_none() && patch.advance.is_none() {
        return Err("请选择至少一项需要修改的时间或推进方式".into());
    }
    let wait = match patch.advance {
        Some(StepAdvance::After { wait_ms }) => Some(wait_ms),
        _ => None,
    };
    if [patch.delay_ms, patch.fade_ms, wait]
        .into_iter()
        .flatten()
        .any(|v| v > stagemaster_playback::MAX_TIME_MS)
    {
        return Err("每项时间须在 0—86400 秒内".into());
    }
    for step in steps
        .iter_mut()
        .filter(|s| selected.contains(text(s, "id")))
    {
        if let Some(delay) = patch.delay_ms {
            step["delay"] = duration(delay);
        }
        if let Some(fade) = patch.fade_ms {
            step["fade"] = duration(fade);
        }
        if let Some(advance) = &patch.advance {
            step["advance"] = match advance {
                StepAdvance::Manual {} => json!({"kind":"manual"}),
                StepAdvance::After { wait_ms } => json!({"kind":"after","wait":duration(*wait_ms)}),
            };
        }
    }
    Ok(())
}
