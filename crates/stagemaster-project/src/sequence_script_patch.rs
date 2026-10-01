//! Sparse human notes; no scheduling, DMX or execution state is changed here.
use crate::{StepScript, sequence_script, text};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepScriptPatch {
    #[serde(default, deserialize_with = "present_text")]
    pub section: Option<String>,
    #[serde(default, deserialize_with = "present_text")]
    pub trigger: Option<String>,
    #[serde(default, deserialize_with = "present_text")]
    pub notes: Option<String>,
}
fn present_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}
impl StepScriptPatch {
    fn merge(&self, mut original: StepScript) -> StepScript {
        if let Some(value) = &self.section {
            original.section.clone_from(value);
        }
        if let Some(value) = &self.trigger {
            original.trigger.clone_from(value);
        }
        if let Some(value) = &self.notes {
            original.notes.clone_from(value);
        }
        original
    }
}
pub(super) fn apply(
    steps: &mut [Value],
    selected: &BTreeSet<&str>,
    patch: &StepScriptPatch,
) -> Result<(), String> {
    if patch.section.is_none() && patch.trigger.is_none() && patch.notes.is_none() {
        return Err("请选择至少一项需要修改的剧本提示".into());
    }
    patch.merge(StepScript::default()).validate()?;
    // Preflight the whole list before changing any step, bounding prepared text to 64 KiB.
    let mut prepared = Vec::new();
    let mut bytes = 0;
    for (index, step) in steps.iter().enumerate() {
        let original = sequence_script::read(step).unwrap_or_default();
        let chosen = selected.contains(text(step, "id"));
        let script = if chosen {
            patch.merge(original)
        } else {
            original
        };
        let script = (!chosen || !script.is_empty()).then_some(script);
        bytes += script.as_ref().map_or(0, StepScript::bytes);
        if bytes > sequence_script::MAX_LIST_BYTES {
            return Err("列表的剧本提示超过 64 KiB，请精简备注或拆分列表".into());
        }
        if chosen {
            prepared.push((index, script));
        }
    }
    for (index, script) in prepared {
        if let Some(script) = script {
            steps[index]["script"] = serde_json::to_value(script).expect("script serialization");
        } else {
            steps[index]
                .as_object_mut()
                .ok_or("步骤无效")?
                .remove("script");
        }
    }
    Ok(())
}
