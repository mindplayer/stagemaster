//! Hold-style fixture programs, separate from continuous effects and timed reset commands.
use crate::{
    FunctionDefinition, FunctionMode, FunctionSelection, ProfileDefault, ProfileDefinition, array,
    text,
};
use serde_json::{Value, json};

pub(crate) const KEY: &str = "fixture-program";
pub(crate) const LABEL: &str = "内置程序";
pub(super) const CAPABILITY: &str = "lighting.fixture-programs";
const EXTERNAL: &str = "external";

pub(super) fn validate_selection(selection: &FunctionSelection) -> Result<(), String> {
    if selection.function_key != EXTERNAL {
        return Err("声控和内置自走档位已屏蔽，演出仅允许外部通道控制".into());
    }
    Ok(())
}

pub(super) fn validate_functions(functions: &[FunctionDefinition]) -> Result<(), String> {
    if !functions.iter().any(|f| f.key == EXTERNAL) {
        return Err("内置程序须定义外部通道控制档位".into());
    }
    for f in functions {
        if f.mode != FunctionMode::Slot
            || !(f.key == EXTERNAL || f.key.starts_with("auto.") || f.key.starts_with("sound."))
        {
            return Err(
                "内置程序仅支持外部控制、自动或声控的固定档位，不支持复位或区间调节".into(),
            );
        }
    }
    Ok(())
}
pub(super) fn validate_default(default: &ProfileDefault) -> Result<(), String> {
    match default {
        ProfileDefault::Function(selection)
            if selection.function_key == EXTERNAL && selection.position == 0 =>
        {
            Ok(())
        }
        _ => Err("内置程序默认须为外部通道控制，不能默认自走或声控".into()),
    }
}
pub(super) fn require(root: &mut Value, definition: &ProfileDefinition) {
    if definition.channels.iter().any(|c| c.attribute == KEY)
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
    let mut present = false;
    for profile in array(&root["lighting"], "profiles") {
        for attribute in array(profile, "attributes")
            .iter()
            .filter(|a| a["key"] == KEY)
        {
            present = true;
            if attribute["valueType"]["kind"] != "function" {
                return Err("内置程序必须使用明确功能选择，不能使用普通百分比".into());
            }
            let channel = array(profile, "channels")
                .iter()
                .find(|c| c["attribute"] == KEY)
                .ok_or("内置程序缺少通道映射")?;
            validate_functions(
                &crate::fixture_value::functions(channel)?.ok_or("内置程序缺少档位")?,
            )?;
            validate_default(&crate::fixture_value::read_default(&attribute["default"])?)?;
        }
    }
    if present
        && !array(root, "requires")
            .iter()
            .any(|r| text(r, "key") == CAPABILITY && r["version"] == 1)
    {
        return Err("工程缺少内置程序能力声明".into());
    }
    Ok(())
}
