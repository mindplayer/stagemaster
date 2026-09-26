//! Reusable editing resources. All mutations run within Document's atomic transaction.
use crate::editing::{find, list, remove, validate_target};
use crate::{array, id, text};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum LibraryEdit {
    SaveGroup {
        id: Option<String>,
        name: String,
        fixture_ids: Vec<String>,
    },
    Duplicate {
        resource: LibraryKind,
        id: String,
        name: String,
    },
    RenamePreset {
        id: String,
        name: String,
    },
    Remove {
        resource: LibraryKind,
        id: String,
        keep_values: bool,
    },
    RecordPreset {
        name: String,
        scene_id: String,
        fixture_ids: Vec<String>,
        attributes: Vec<String>,
    },
    UpdatePreset {
        id: String,
        scene_id: String,
        fixture_ids: Vec<String>,
        attributes: Vec<String>,
        mode: PresetUpdate,
    },
    ApplyPreset {
        id: String,
        scene_id: String,
        fixture_ids: Vec<String>,
        attributes: Vec<String>,
        linked: bool,
    },
    Detach {
        scene_id: String,
        fixture_ids: Vec<String>,
        attributes: Vec<String>,
    },
    CopyValues {
        scene_id: String,
        source_id: String,
        fixture_ids: Vec<String>,
        attributes: Vec<String>,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LibraryKind {
    Group,
    Preset,
}
impl LibraryKind {
    fn key(&self) -> &'static str {
        match self {
            Self::Group => "groups",
            Self::Preset => "presets",
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresetUpdate {
    Existing,
    Merge,
    Replace,
}

pub(super) fn apply(root: &mut Value, command: LibraryEdit) -> Result<(), String> {
    match command {
        LibraryEdit::SaveGroup {
            id: existing,
            name,
            fixture_ids,
        } => {
            validate_fixtures(root, &fixture_ids)?;
            if let Some(existing) = existing {
                let group = find(list(root, "groups")?, &existing)?;
                group["name"] = name.into();
                group["fixtureIds"] = json!(fixture_ids);
            } else {
                list(root, "groups")?.push(json!({"id":id(),"name":name,"fixtureIds":fixture_ids}));
            }
        }
        LibraryEdit::Duplicate {
            resource,
            id: source,
            name,
        } => {
            let mut copy = find(list(root, resource.key())?, &source)?.clone();
            copy["id"] = id().into();
            copy["name"] = name.into();
            list(root, resource.key())?.push(copy);
        }
        LibraryEdit::RenamePreset { id, name } => {
            find(list(root, "presets")?, &id)?["name"] = name.into();
        }
        LibraryEdit::Remove {
            resource,
            id,
            keep_values,
        } => {
            if matches!(resource, LibraryKind::Preset) {
                remove_preset(root, &id, keep_values)?;
            } else {
                remove(list(root, resource.key())?, &id)?;
            }
        }
        LibraryEdit::RecordPreset {
            name,
            scene_id,
            fixture_ids,
            attributes,
        } => {
            let values = capture(root, &scene_id, &fixture_ids, &attributes)?;
            list(root, "presets")?.push(json!({"id":id(),"name":name,"values":values}));
        }
        LibraryEdit::UpdatePreset {
            id,
            scene_id,
            fixture_ids,
            attributes,
            mode,
        } => {
            let values = capture(root, &scene_id, &fixture_ids, &attributes)?;
            update_preset(root, &id, values, mode)?;
        }
        LibraryEdit::ApplyPreset {
            id,
            scene_id,
            fixture_ids,
            attributes,
            linked,
        } => {
            validate_selection(root, &fixture_ids, &attributes)?;
            let values: Vec<Value> = array(find(list(root, "presets")?, &id)?, "values").to_vec();
            let values = values
                .into_iter()
                .filter(|v| selected(v, &fixture_ids, &attributes));
            let assignments: Vec<Value> = values
                .map(|v| {
                    json!({"target":v["target"],"operation":"set","source":
                        if linked { json!({"kind":"preset","presetId":id}) }
                        else { json!({"kind":"literal","value":v["value"]}) }
                    })
                })
                .collect();
            if assignments.is_empty() {
                return Err("此预设没有匹配所选灯具和属性的内容".into());
            }
            install(root, &scene_id, assignments)?;
        }
        LibraryEdit::Detach {
            scene_id,
            fixture_ids,
            attributes,
        } => {
            detach(root, &scene_id, &fixture_ids, &attributes)?;
        }
        LibraryEdit::CopyValues {
            scene_id,
            source_id,
            fixture_ids,
            attributes,
        } => {
            copy_values(root, &scene_id, &source_id, &fixture_ids, &attributes)?;
        }
    }
    Ok(())
}

fn validate_fixtures(root: &Value, ids: &[String]) -> Result<(), String> {
    unique_nonempty(ids, "灯具")?;
    for id in ids {
        if !array(&root["lighting"], "fixtures")
            .iter()
            .any(|f| f["id"] == *id)
        {
            return Err("所选灯具已不存在，请重新选择".into());
        }
    }
    Ok(())
}
fn unique_nonempty(values: &[String], label: &str) -> Result<(), String> {
    if values.is_empty()
        || values.len() > 10000
        || values.iter().any(|v| v.trim().is_empty())
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return Err(format!("请选择 1–10000 个不重复的{label}"));
    }
    Ok(())
}
fn validate_selection(root: &Value, ids: &[String], attributes: &[String]) -> Result<(), String> {
    validate_fixtures(root, ids)?;
    unique_nonempty(attributes, "属性")?;
    for attribute in attributes {
        if !ids
            .iter()
            .any(|id| validate_target(root, id, attribute).is_ok())
        {
            return Err(format!("所选灯具均不支持属性 {attribute}"));
        }
    }
    Ok(())
}
fn selected(entry: &Value, fixtures: &[String], attributes: &[String]) -> bool {
    fixtures
        .iter()
        .any(|id| entry["target"]["fixtureId"] == *id)
        && attributes
            .iter()
            .any(|key| entry["target"]["attribute"] == *key)
}
fn scene<'a>(root: &'a Value, id: &str) -> Result<&'a Value, String> {
    array(&root["lighting"], "scenes")
        .iter()
        .find(|s| s["id"] == id)
        .ok_or_else(|| "来源场景不存在".into())
}
/// One resolver is shared by view, compilation and resource editing; no nested references.
pub(super) fn resolved_value(lighting: &Value, entry: &Value) -> Option<Value> {
    if entry["operation"] != "set" {
        return None;
    }
    let source = &entry["source"];
    if source["kind"] == "literal" {
        return Some(source["value"].clone());
    }
    array(lighting, "presets")
        .iter()
        .find(|p| p["id"] == source["presetId"])
        .and_then(|p| {
            array(p, "values")
                .iter()
                .find(|v| v["target"] == entry["target"])
        })
        .map(|v| v["value"].clone())
}
fn capture(
    root: &Value,
    scene_id: &str,
    fixtures: &[String],
    attributes: &[String],
) -> Result<Vec<Value>, String> {
    validate_selection(root, fixtures, attributes)?;
    let scene = scene(root, scene_id)?;
    let values: Vec<Value> = array(scene, "assignments")
        .iter()
        .filter(|a| selected(a, fixtures, attributes))
        .filter_map(|a| {
            resolved_value(&root["lighting"], a).map(|v| json!({"target":a["target"],"value":v}))
        })
        .collect();
    if values.is_empty() {
        return Err("所选属性没有已记录的数值；释放和未记录项不能存入预设或复制".into());
    }
    Ok(values)
}
fn literal(target: &Value, value: &Value) -> Value {
    json!({"target":target,"operation":"set","source":{"kind":"literal","value":value}})
}
fn install(root: &mut Value, scene_id: &str, assignments: Vec<Value>) -> Result<(), String> {
    let scene = find(list(root, "scenes")?, scene_id)?;
    let entries = scene["assignments"].as_array_mut().ok_or("场景属性无效")?;
    for assignment in assignments {
        if let Some(entry) = entries
            .iter_mut()
            .find(|a| a["target"] == assignment["target"])
        {
            *entry = assignment;
        } else {
            entries.push(assignment);
        }
    }
    Ok(())
}
fn update_preset(
    root: &mut Value,
    id: &str,
    values: Vec<Value>,
    mode: PresetUpdate,
) -> Result<(), String> {
    let old = array(find(list(root, "presets")?, id)?, "values").to_vec();
    let mut next = if matches!(mode, PresetUpdate::Replace) {
        Vec::new()
    } else {
        old.clone()
    };
    let mut matched = false;
    for value in values {
        if let Some(entry) = next.iter_mut().find(|v| v["target"] == value["target"]) {
            *entry = value;
            matched = true;
        } else if !matches!(mode, PresetUpdate::Existing) {
            next.push(value);
            matched = true;
        }
    }
    if !matched {
        return Err("当前选择与预设已有内容没有交集，请改用合并新增".into());
    }
    for scene in array(&root["lighting"], "scenes") {
        for a in array(scene, "assignments") {
            if a["source"]["presetId"] == id && !next.iter().any(|v| v["target"] == a["target"]) {
                return Err(format!(
                    "场景“{}”仍引用将被移除的预设属性；请先解除引用或使用合并",
                    text(scene, "name")
                ));
            }
        }
    }
    find(list(root, "presets")?, id)?["values"] = next.into();
    Ok(())
}
fn remove_preset(root: &mut Value, id: &str, keep: bool) -> Result<(), String> {
    let values = array(find(list(root, "presets")?, id)?, "values").to_vec();
    for scene in list(root, "scenes")? {
        let name = text(scene, "name").to_owned();
        for a in scene["assignments"].as_array_mut().ok_or("场景属性无效")? {
            if a["source"]["presetId"] == id {
                if !keep {
                    return Err(format!(
                        "场景“{name}”仍引用此预设，可选择保留场景数值并删除"
                    ));
                }
                let value = values
                    .iter()
                    .find(|v| v["target"] == a["target"])
                    .ok_or("预设引用缺失")?;
                *a = literal(&a["target"], &value["value"]);
            }
        }
    }
    remove(list(root, "presets")?, id)
}

fn detach(
    root: &mut Value,
    scene_id: &str,
    fixture_ids: &[String],
    attributes: &[String],
) -> Result<(), String> {
    validate_selection(root, fixture_ids, attributes)?;
    let scene = scene(root, scene_id)?;
    let assignments: Vec<Value> = array(scene, "assignments")
        .iter()
        .filter(|a| a["source"]["kind"] == "preset" && selected(a, fixture_ids, attributes))
        .map(|a| {
            resolved_value(&root["lighting"], a)
                .map(|v| literal(&a["target"], &v))
                .ok_or("预设引用缺失")
        })
        .collect::<Result<_, _>>()?;
    if assignments.is_empty() {
        return Err("所选属性没有预设引用".into());
    }
    install(root, scene_id, assignments)
}
fn copy_values(
    root: &mut Value,
    scene_id: &str,
    source_id: &str,
    fixture_ids: &[String],
    attributes: &[String],
) -> Result<(), String> {
    validate_selection(root, fixture_ids, attributes)?;
    for attribute in attributes {
        validate_target(root, source_id, attribute)?;
    }
    let values = capture(root, scene_id, &[source_id.into()], attributes)?;
    if values.len() * fixture_ids.len() > 10000 {
        return Err("本次复制超过 10000 项属性，请缩小选择范围".into());
    }
    let mut assignments = Vec::new();
    for fixture in fixture_ids {
        for attribute in attributes {
            validate_target(root, fixture, attribute)?;
        }
        for value in &values {
            let target = json!({"fixtureId":fixture,"attribute":value["target"]["attribute"]});
            assignments.push(literal(&target, &value["value"]));
        }
    }
    install(root, scene_id, assignments)
}
