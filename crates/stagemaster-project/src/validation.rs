use crate::{array, text};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

fn schema() -> &'static jsonschema::Validator {
    static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let common = include_str!("../../../docs/project-format/schemas/common.schema.json");
        let project = include_str!("../../../docs/project-format/schemas/project.schema.json");
        // Keep the checked-in schemas authoritative; resolve their URN reference in memory.
        let rewrite = |s: &str| {
            s.replace(
                "urn:stagemaster:schema:common:0.1.0-draft.1#/$defs/",
                "#/$defs/Common/$defs/",
            )
        };
        let mut root: Value =
            serde_json::from_str(&rewrite(project)).expect("embedded project schema");
        let mut common: Value =
            serde_json::from_str(&rewrite(common)).expect("embedded common schema");
        common.as_object_mut().expect("schema object").remove("$id");
        root["$defs"]["Common"] = common;
        jsonschema::validator_for(&root).expect("embedded schemas must compile")
    })
}

pub(super) fn validate(root: &Value) -> Result<(), String> {
    schema()
        .validate(root)
        .map_err(|error| format!("工程字段不符合格式要求：{}", error.instance_path()))?;
    if array(&root["project"], "parentRevisionIds").contains(&root["project"]["revisionId"]) {
        return Err("工程修订不能引用自身作为父修订".into());
    }
    supported(root)?;
    unique_objects(root, &mut BTreeSet::new())?;
    crate::stage::validate(root)?;
    let Some(lighting) = root.get("lighting") else {
        return Ok(());
    };
    let domains = indexed(array(root, "domains"));
    let profiles = indexed(array(lighting, "profiles"));
    let fixtures = indexed(array(lighting, "fixtures"));
    let presets = indexed(array(lighting, "presets"));
    for profile in profiles.values() {
        validate_profile(profile)?;
    }
    for fixture in fixtures.values() {
        lookup(&profiles, text(fixture, "profileId"), "灯具档案")?;
        lookup(&domains, text(fixture, "domainId"), "灯光输出域")?;
    }
    validate_patches(lighting, &fixtures, &profiles)?;
    for group in array(lighting, "groups") {
        for id in array(group, "fixtureIds") {
            lookup(&fixtures, id.as_str().unwrap_or_default(), "灯组成员")?;
        }
    }
    for preset in presets.values() {
        let mut targets = BTreeSet::new();
        for entry in array(preset, "values") {
            validate_target(&entry["target"], &fixtures, &profiles)?;
            unique_target(&entry["target"], &mut targets)?;
            normalized(&entry["value"])?;
        }
    }
    for scene in array(lighting, "scenes") {
        let mut targets = BTreeSet::new();
        for entry in array(scene, "assignments") {
            validate_target(&entry["target"], &fixtures, &profiles)?;
            unique_target(&entry["target"], &mut targets)?;
            if entry["operation"] == "release" {
                continue;
            }
            if entry["source"]["kind"] == "literal" {
                normalized(&entry["source"]["value"])?;
            } else {
                let preset = lookup(&presets, text(&entry["source"], "presetId"), "场景预设")?;
                if !array(preset, "values")
                    .iter()
                    .any(|value| value["target"] == entry["target"])
                {
                    return Err(format!(
                        "场景“{}”引用的预设没有对应灯具属性",
                        text(scene, "name")
                    ));
                }
            }
        }
    }
    crate::sequence::validate(root)?;
    Ok(())
}

fn supported(root: &Value) -> Result<(), String> {
    for key in [
        "resources",
        "syncGroups",
        "actions",
        "conditions",
        "rules",
        "timelines",
        "entryPoints",
        "extensions",
    ] {
        if !array(root, key).is_empty() {
            return Err(format!("当前版本尚不支持此工程中的 {key} 内容，工程未打开"));
        }
    }
    for key in ["media", "motion", "io", "monitoring", "surfaces"] {
        if root.get(key).is_some() {
            return Err(format!("当前版本尚不支持此工程中的 {key} 模块，工程未打开"));
        }
    }
    let mut capabilities = BTreeSet::new();
    for capability in array(root, "requires") {
        if !["lighting.basic", "stage.layout", "stage.spaces"].contains(&text(capability, "key"))
            || capability["version"] != 1
        {
            return Err(format!(
                "当前版本不支持工程能力：{}",
                text(capability, "key")
            ));
        }
        if !capabilities.insert(text(capability, "key")) {
            return Err("工程能力声明重复".into());
        }
    }
    if root.get("lighting").is_some() && !capabilities.contains("lighting.basic") {
        return Err("灯光工程缺少灯光能力声明".into());
    }
    if root.get("stage").is_some()
        && (!capabilities.contains("stage.layout") || !capabilities.contains("stage.spaces"))
    {
        return Err("场地工程缺少空间能力声明".into());
    }
    if array(root, "domains")
        .iter()
        .any(|domain| domain["kind"] != "lighting")
    {
        return Err("当前版本仅支持灯光输出域".into());
    }
    Ok(())
}

fn unique_objects<'a>(value: &'a Value, ids: &mut BTreeSet<&'a str>) -> Result<(), String> {
    if let Some(object) = value.as_object() {
        if let Some(id) = object.get("id").and_then(Value::as_str)
            && !ids.insert(id)
        {
            return Err(format!("工程对象身份重复：{id}"));
        }
        if object
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name.trim().is_empty())
        {
            return Err("名称不能全为空格".into());
        }
        for child in object.values() {
            unique_objects(child, ids)?;
        }
    } else if let Some(list) = value.as_array() {
        for child in list {
            unique_objects(child, ids)?;
        }
    }
    Ok(())
}

fn indexed(values: &[Value]) -> BTreeMap<&str, &Value> {
    values
        .iter()
        .map(|value| (text(value, "id"), value))
        .collect()
}
fn lookup<'a>(
    index: &BTreeMap<&str, &'a Value>,
    id: &str,
    kind: &str,
) -> Result<&'a Value, String> {
    index
        .get(id)
        .copied()
        .ok_or_else(|| format!("{kind}引用不存在或类型不符：{id}"))
}
fn normalized(value: &Value) -> Result<(), String> {
    if value["kind"] != "normalized" {
        return Err("当前灯光编辑仅支持归一化属性".into());
    }
    Ok(())
}
fn validate_profile(profile: &Value) -> Result<(), String> {
    if profile.get("sourceResourceId").is_some() {
        return Err("当前版本不支持外部灯具档案资源".into());
    }
    let mut attributes = BTreeSet::new();
    for attribute in array(profile, "attributes") {
        normalized(&attribute["valueType"])?;
        normalized(&attribute["default"])?;
        if !attributes.insert(text(attribute, "key")) {
            return Err("灯具档案属性重复".into());
        }
    }
    let mut mapped = BTreeSet::new();
    let mut occupied = BTreeSet::new();
    for channel in array(profile, "channels") {
        let key = text(channel, "attribute");
        let expected = if channel["encoding"] == "u8" { 1 } else { 2 };
        if !attributes.contains(key)
            || !mapped.insert(key)
            || array(channel, "offsets").len() != expected
        {
            return Err("灯具档案通道映射无效".into());
        }
        for offset in array(channel, "offsets") {
            let offset = offset.as_u64().unwrap_or_default();
            if offset >= profile["footprint"].as_u64().unwrap_or_default()
                || !occupied.insert(offset)
            {
                return Err("灯具档案通道越界或重叠".into());
            }
        }
    }
    if mapped != attributes {
        return Err("灯具档案存在未映射属性".into());
    }
    Ok(())
}
fn validate_target(
    target: &Value,
    fixtures: &BTreeMap<&str, &Value>,
    profiles: &BTreeMap<&str, &Value>,
) -> Result<(), String> {
    let fixture = lookup(fixtures, text(target, "fixtureId"), "属性目标灯具")?;
    let profile = lookup(profiles, text(fixture, "profileId"), "灯具档案")?;
    if !array(profile, "attributes")
        .iter()
        .any(|attribute| attribute["key"] == target["attribute"])
    {
        return Err(format!(
            "灯具“{}”没有属性 {}",
            text(fixture, "name"),
            text(target, "attribute")
        ));
    }
    Ok(())
}
fn unique_target<'a>(
    target: &'a Value,
    seen: &mut BTreeSet<(&'a str, &'a str)>,
) -> Result<(), String> {
    if !seen.insert((text(target, "fixtureId"), text(target, "attribute"))) {
        return Err("同一记录中灯具属性重复".into());
    }
    Ok(())
}
fn validate_patches(
    lighting: &Value,
    fixtures: &BTreeMap<&str, &Value>,
    profiles: &BTreeMap<&str, &Value>,
) -> Result<(), String> {
    let mut patched = BTreeSet::new();
    let mut ranges: BTreeMap<(&str, u64), Vec<(u64, u64)>> = BTreeMap::new();
    for patch in array(lighting, "patches") {
        let fixture = lookup(fixtures, text(patch, "fixtureId"), "配适灯具")?;
        if !patched.insert(text(patch, "fixtureId")) || patch["domainId"] != fixture["domainId"] {
            return Err("灯具配适重复或输出域不一致".into());
        }
        let profile = lookup(profiles, text(fixture, "profileId"), "灯具档案")?;
        let start = patch["address"].as_u64().unwrap_or_default();
        let end = start + profile["footprint"].as_u64().unwrap_or_default() - 1;
        if end > 512 {
            return Err(format!("灯具“{}”超出 512 通道", text(fixture, "name")));
        }
        let group = ranges
            .entry((
                text(patch, "domainId"),
                patch["universe"].as_u64().unwrap_or_default(),
            ))
            .or_default();
        if group.iter().any(|&(a, b)| start <= b && end >= a) {
            return Err(format!(
                "灯具“{}”的配适地址与其他灯具重叠",
                text(fixture, "name")
            ));
        }
        group.push((start, end));
    }
    Ok(())
}
