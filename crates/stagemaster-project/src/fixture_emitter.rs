//! Stable, bounded light-source ownership. No virtual fixtures, optical model or clock.
use crate::{ProfileDefinition, array, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const CAPABILITY: &str = "lighting.fixture-emitters";
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmitterDefinition {
    pub key: String,
    pub name: String,
}
pub(super) fn split(key: &str) -> Option<(&str, &str)> {
    let (owner, attribute) = key.strip_prefix("emitter.")?.split_once('.')?;
    Some((owner, attribute))
}
pub(super) fn base(key: &str) -> &str {
    split(key).map_or(key, |(_, attribute)| attribute)
}
fn valid_key(key: &str) -> bool {
    (1..=32).contains(&key.len())
        && !key.ends_with('-')
        && !key.contains("--")
        && key.as_bytes()[0].is_ascii_lowercase()
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn linear_set(mut keys: Vec<&str>) -> bool {
    keys.sort_unstable();
    [
        vec!["dimmer"],
        vec!["blue", "green", "red"],
        vec!["blue", "green", "red", "white"],
        vec!["blue", "dimmer", "green", "red"],
        vec!["blue", "dimmer", "green", "red", "white"],
    ]
    .contains(&keys)
}
pub(super) fn supported<'a>(keys: impl Iterator<Item = &'a str>) -> bool {
    let mut units: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut root = Vec::new();
    let mut unique = BTreeSet::new();
    for key in keys {
        if !unique.insert(key) {
            return false;
        }
        if let Some((owner, attribute)) = split(key) {
            if !valid_key(owner) {
                return false;
            }
            units.entry(owner).or_default().push(attribute);
        } else if key.starts_with("emitter.") {
            return false;
        } else {
            root.push(key);
        }
    }
    if units.is_empty() {
        return crate::fixture::supported_root_keys(root.into_iter());
    }
    if units.len() > 32
        || !units.values().all(|keys| linear_set(keys.clone()))
        || root
            .iter()
            .any(|key| ["red", "green", "blue", "white"].contains(key))
    {
        return false;
    }
    // The old validator expects one intensity family. Here a real master is optional.
    if !root.contains(&"dimmer") {
        root.push("dimmer");
    }
    crate::fixture::supported_root_keys(root.into_iter())
}
fn owners(emitters: &[EmitterDefinition], keys: &[&str]) -> Result<(), String> {
    let scoped = keys
        .iter()
        .filter(|key| key.starts_with("emitter."))
        .count();
    if emitters.is_empty() && scoped == 0 {
        return Ok(());
    }
    if !(1..=32).contains(&emitters.len()) {
        return Err("独立光源需要声明 1–32 个单元".into());
    }
    let mut declared = BTreeSet::new();
    for emitter in emitters {
        if !valid_key(&emitter.key) || !declared.insert(emitter.key.as_str()) {
            return Err("光源标识须唯一，使用 1–32 位小写字母、数字或连字符，以字母开头".into());
        }
        if emitter.name.trim().is_empty() || emitter.name.chars().count() > 64 {
            return Err("光源名称需要 1–64 个字符".into());
        }
    }
    let used = keys
        .iter()
        .filter_map(|key| split(key).map(|(owner, _)| owner))
        .collect::<BTreeSet<_>>();
    if declared != used || !supported(keys.iter().copied()) {
        return Err("光源属性归属无效；每单元须具备调光、完整 RGB 或 RGBW，可另带单元调光".into());
    }
    Ok(())
}
pub(super) fn validate_definition(definition: &ProfileDefinition) -> Result<(), String> {
    let emitters = definition.emitters.as_deref().unwrap_or_default();
    if definition.emitters.is_some() && emitters.is_empty() {
        return Err("独立光源列表不能为空".into());
    }
    owners(
        emitters,
        &definition
            .channels
            .iter()
            .map(|c| c.attribute.as_str())
            .collect::<Vec<_>>(),
    )?;
    for channel in &definition.channels {
        if channel.attribute.starts_with("emitter.")
            && (channel.functions.is_some()
                || !matches!(channel.default_value, crate::ProfileDefault::Normalized(_)))
        {
            return Err(
                "本版本独立光源只支持全范围线性控制，不支持功能、声控、自走或复位宏".into(),
            );
        }
    }
    Ok(())
}
pub(super) fn require(root: &mut Value, definition: &ProfileDefinition) {
    if definition.emitters.is_some()
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
    let declared = array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY && r["version"] == 1);
    for profile in array(&root["lighting"], "profiles") {
        let emitters: Option<Vec<EmitterDefinition>> = profile
            .get("emitters")
            .map(|v| serde_json::from_value(v.clone()).map_err(|_| "光源定义字段无效".to_string()))
            .transpose()?;
        let keys = array(profile, "attributes")
            .iter()
            .map(|a| text(a, "key"))
            .collect::<Vec<_>>();
        if emitters.is_none() && !keys.iter().any(|k| k.starts_with("emitter.")) {
            continue;
        }
        if !declared {
            return Err("工程缺少独立光源能力声明".into());
        }
        if emitters.as_ref().is_some_and(Vec::is_empty) {
            return Err("独立光源列表不能为空".into());
        }
        owners(emitters.as_deref().unwrap_or_default(), &keys)?;
        if let Some(master) = array(profile, "attributes")
            .iter()
            .find(|a| a["key"] == "dimmer")
            && (master["valueType"]["kind"] != "normalized" || master["mix"] != "htp")
        {
            return Err("独立光源总调光须为线性值并采用高值优先".into());
        }
        for attribute in array(profile, "attributes")
            .iter()
            .filter(|a| text(a, "key").starts_with("emitter."))
        {
            let key = text(attribute, "key");
            let channel = array(profile, "channels")
                .iter()
                .find(|c| c["attribute"] == key)
                .ok_or("光源属性缺少通道")?;
            if attribute["valueType"]["kind"] != "normalized"
                || channel.get("functions").is_some()
                || attribute["mix"] != if base(key) == "dimmer" { "htp" } else { "ltp" }
            {
                return Err("光源属性须为全范围线性值，调光高值优先、颜色后值优先".into());
            }
        }
    }
    Ok(())
}
pub(super) fn label(profile: &Value, key: &str) -> String {
    if key == "dimmer" && profile.get("emitters").is_some() {
        return "总亮度".into();
    }
    if let Some((owner, attribute)) = split(key) {
        let name = array(profile, "emitters")
            .iter()
            .find(|e| e["key"] == owner)
            .map_or(owner, |e| text(e, "name"));
        format!("{name} · {}", crate::view::attribute_label(attribute))
    } else {
        crate::view::attribute_label(key).into()
    }
}
