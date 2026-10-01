//! Project adapters for typed functions. Only this layer lowers semantic selections to DMX values.
use crate::{FunctionDefinition, FunctionSelection, FunctionTable, ProfileDefault, array, text};
use serde_json::{Value, json};

pub(super) const CAPABILITY: &str = "lighting.fixture-functions";
pub(super) fn is_function_key(key: &str) -> bool {
    ["color-wheel", "gobo-wheel", "shutter", "prism"].contains(&key)
}
pub(super) fn require(root: &mut Value) {
    if !array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .expect("validated requires")
            .push(json!({"key":CAPABILITY,"version":1}));
    }
}
pub(super) fn stored_default(value: &ProfileDefault) -> Value {
    match value {
        ProfileDefault::Normalized(value) => json!({"kind":"normalized","value":value}),
        ProfileDefault::Function(selection) => stored_selection(selection),
    }
}
pub(super) fn stored_selection(selection: &FunctionSelection) -> Value {
    json!({"kind":"function","functionKey":selection.function_key,"position":selection.position})
}
pub(super) fn selection(value: &Value) -> Result<FunctionSelection, String> {
    if value["kind"] != "function" {
        return Err("功能属性需要明确的功能选择，不能使用普通百分比".into());
    }
    Ok(FunctionSelection {
        function_key: value["functionKey"].as_str().ok_or("功能标识无效")?.into(),
        position: value["position"]
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .ok_or("功能区间位置无效")?,
    })
}
pub(super) fn read_default(value: &Value) -> Result<ProfileDefault, String> {
    if value["kind"] == "function" {
        return selection(value).map(ProfileDefault::Function);
    }
    if value["kind"] != "normalized" {
        return Err("不支持此灯具属性类型".into());
    }
    value["value"]
        .as_u64()
        .and_then(|v| u16::try_from(v).ok())
        .map(ProfileDefault::Normalized)
        .ok_or_else(|| "属性值超出范围".into())
}
pub(super) fn functions(channel: &Value) -> Result<Option<Vec<FunctionDefinition>>, String> {
    channel
        .get("functions")
        .map(|v| serde_json::from_value(v.clone()).map_err(|_| "功能区间字段无效".into()))
        .transpose()
}
pub(super) fn profile<'a>(lighting: &'a Value, fixture_id: &str) -> Result<&'a Value, String> {
    let fixture = array(lighting, "fixtures")
        .iter()
        .find(|f| f["id"] == fixture_id)
        .ok_or("灯具不存在")?;
    array(lighting, "profiles")
        .iter()
        .find(|p| p["id"] == fixture["profileId"])
        .ok_or_else(|| "灯具档案不存在".into())
}
pub(super) fn encode(profile: &Value, key: &str, value: &Value) -> Result<u16, String> {
    let attribute = array(profile, "attributes")
        .iter()
        .find(|a| a["key"] == key)
        .ok_or("灯具属性不存在")?;
    let channel = array(profile, "channels")
        .iter()
        .find(|c| c["attribute"] == key)
        .ok_or("灯具属性没有通道映射")?;
    if attribute["valueType"]["kind"] == "function" {
        if !is_function_key(key) || attribute["mix"] != "ltp" {
            return Err("功能属性仅支持色盘、图案盘、快门和棱镜，并采用后值优先".into());
        }
        let functions = functions(channel)?.ok_or("功能属性缺少区间定义")?;
        crate::fixture_appearance::validate_channel(key, &functions)?;
        FunctionTable::new(&functions, channel["encoding"] == "u16-be")?.encode(&selection(value)?)
    } else {
        if channel.get("functions").is_some() {
            return Err("普通属性不能携带功能区间".into());
        }
        if attribute["valueType"]["kind"] != "normalized" {
            return Err("当前版本不支持此灯具属性类型".into());
        }
        match read_default(value)? {
            ProfileDefault::Normalized(value) => Ok(value),
            ProfileDefault::Function(_) => Err("普通属性不能接收功能选择".into()),
        }
    }
}
/// Validate defaults and require an explicit capability even for unused profiles.
pub(super) fn validate(root: &Value) -> Result<(), String> {
    crate::fixture_appearance::validate(root)?;
    let mut present = false;
    for profile in array(&root["lighting"], "profiles") {
        for attribute in array(profile, "attributes") {
            present |= attribute["valueType"]["kind"] == "function";
            encode(profile, text(attribute, "key"), &attribute["default"])?;
        }
    }
    if present
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY && r["version"] == 1)
    {
        return Err("工程缺少灯具功能区间能力声明".into());
    }
    Ok(())
}

pub(super) fn set_scene(
    root: &mut Value,
    scene_id: &str,
    fixture_id: &str,
    attribute: &str,
    selection: &FunctionSelection,
) -> Result<(), String> {
    let value = stored_selection(selection);
    encode(profile(&root["lighting"], fixture_id)?, attribute, &value)?;
    crate::editing::set_scene_entry(
        root,
        scene_id,
        fixture_id,
        attribute,
        crate::ValueMode::Literal,
        &value,
    )
}
