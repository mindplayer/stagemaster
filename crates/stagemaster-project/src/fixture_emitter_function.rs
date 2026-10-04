//! Restricted, explicitly controlled functions of a light source; not autonomous programs.
use crate::{FunctionDefinition, FunctionMode, ProfileDefinition, array, text};
use serde_json::{Value, json};
pub(super) const CAPABILITY: &str = "lighting.fixture-emitter-functions";
pub(super) fn supported(key: &str) -> bool {
    crate::fixture_emitter::split(key)
        .is_some_and(|(_, base)| ["shutter", "color-wheel", "gobo-wheel"].contains(&base))
}
pub(super) fn validate_channel(key: &str, functions: &[FunctionDefinition]) -> Result<(), String> {
    let Some((_, base)) = crate::fixture_emitter::split(key) else {
        return Ok(());
    };
    for f in functions {
        let slot = f.mode == FunctionMode::Slot;
        let allowed = match base {
            "shutter" => match f.key.as_str() {
                "open" | "closed" => slot,
                "strobe" => !slot,
                _ => false,
            },
            "color-wheel" | "gobo-wheel" => {
                ((f.key == "open" || (f.key.starts_with("slot-") && f.key.len() > 5)) && slot)
                    || (base == "gobo-wheel"
                        && f.key.starts_with("shake-")
                        && f.key.len() > 6
                        && !slot)
            }
            _ => false,
        };
        if !allowed {
            return Err("独立光源仅允许开闭、受控频闪、固定轮盘或单图案抖动；声控、自走、复位及未知宏不可执行".into());
        }
    }
    Ok(())
}
pub(super) fn require(root: &mut Value, definition: &ProfileDefinition) {
    if definition.channels.iter().any(|c| supported(&c.attribute))
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .expect("validated requires")
            .push(json!({"key":CAPABILITY,"version":1}));
    }
}
pub(super) fn validate(root: &Value) -> Result<(), String> {
    let present = array(&root["lighting"], "profiles").iter().any(|p| {
        array(p, "attributes")
            .iter()
            .any(|a| supported(text(a, "key")))
    });
    if present
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY && r["version"] == 1)
    {
        return Err("工程缺少独立光源功能能力声明".into());
    }
    Ok(())
}
