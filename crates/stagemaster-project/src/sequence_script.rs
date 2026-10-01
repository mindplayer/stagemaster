//! Human show-call notes. These never become scheduling or output instructions.
use crate::{array, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const CAPABILITY: &str = "lighting.sequence-script";
pub(super) const MAX_LIST_BYTES: usize = 64 * 1024;
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepScript {
    pub section: String,
    pub trigger: String,
    pub notes: String,
}
impl StepScript {
    pub(super) fn validate(&self) -> Result<(), String> {
        for (label, value, limit) in [
            ("幕／场", &self.section, 80),
            ("台词／动作提示", &self.trigger, 1024),
            ("排练备注", &self.notes, 4096),
        ] {
            if value.chars().count() > limit
                || value
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            {
                return Err(format!(
                    "{label}最多 {limit} 个字符，且不能包含不可见控制字符"
                ));
            }
        }
        Ok(())
    }
    pub(super) fn is_empty(&self) -> bool {
        [&self.section, &self.trigger, &self.notes]
            .iter()
            .all(|v| v.trim().is_empty())
    }
    pub(super) fn bytes(&self) -> usize {
        self.section.len() + self.trigger.len() + self.notes.len()
    }
}
pub(super) fn read(step: &Value) -> Option<StepScript> {
    step.get("script")
        .map(|s| serde_json::from_value(s.clone()).expect("validated script"))
}
pub(super) fn set(
    root: &mut Value,
    sequence_id: &str,
    step_id: &str,
    script: Option<StepScript>,
) -> Result<(), String> {
    if let Some(script) = &script {
        script.validate()?;
    }
    let sequence = crate::editing::find(crate::editing::list(root, "sequences")?, sequence_id)?;
    let step = crate::editing::find(
        sequence["steps"].as_array_mut().ok_or("列表步骤无效")?,
        step_id,
    )?;
    if let Some(script) = script.filter(|s| !s.is_empty()) {
        step["script"] = serde_json::to_value(script).expect("script serialization");
    } else {
        step.as_object_mut().ok_or("步骤无效")?.remove("script");
    }
    Ok(())
}
pub(super) fn sync_capability(root: &mut Value) {
    let used = array(&root["lighting"], "sequences")
        .iter()
        .any(|s| array(s, "steps").iter().any(|s| s.get("script").is_some()));
    let requires = root["requires"]
        .as_array_mut()
        .expect("validated capabilities");
    if used {
        if !requires.iter().any(|c| c["key"] == CAPABILITY) {
            requires.push(json!({"key":CAPABILITY,"version":1}));
        }
    } else {
        requires.retain(|c| c["key"] != CAPABILITY);
    }
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let declared = array(root, "requires")
        .iter()
        .any(|c| c["key"] == CAPABILITY && c["version"] == 1);
    for sequence in array(&root["lighting"], "sequences") {
        let mut bytes = 0;
        for step in array(sequence, "steps") {
            if let Some(script) = read(step) {
                if !declared {
                    return Err("工程缺少剧本提示能力声明".into());
                }
                script
                    .validate()
                    .map_err(|e| format!("步骤“{}”：{e}", text(step, "name")))?;
                bytes += script.bytes();
            }
        }
        if bytes > MAX_LIST_BYTES {
            return Err(format!(
                "列表“{}”的剧本提示超过 64 KiB，请精简备注或拆分列表",
                text(sequence, "name")
            ));
        }
    }
    Ok(())
}
