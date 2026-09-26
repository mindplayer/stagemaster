use crate::{array, id, text};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum EditCommand {
    Library {
        command: crate::LibraryEdit,
    },
    Sequence {
        command: crate::SequenceEdit,
    },
    Batch {
        commands: Vec<EditCommand>,
    },
    SetInfo {
        name: String,
        description: String,
    },
    AddFixture {
        name: String,
        profile_id: String,
        domain_id: String,
        universe: u16,
        address: u16,
    },
    UpdateFixture {
        id: String,
        name: String,
        universe: u16,
        address: u16,
    },
    RemoveFixture {
        id: String,
    },
    AddScene {
        name: String,
    },
    DuplicateScene {
        id: String,
        name: String,
    },
    RenameScene {
        id: String,
        name: String,
    },
    RemoveScene {
        id: String,
    },
    SetSceneValue {
        scene_id: String,
        fixture_id: String,
        attribute: String,
        mode: ValueMode,
        value: u16,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueMode {
    Literal,
    Release,
    Remove,
}

pub(super) fn apply(root: &mut Value, command: EditCommand) -> Result<(), String> {
    match command {
        EditCommand::Library { command } => crate::library::apply(root, command)?,
        EditCommand::Sequence { command } => crate::sequence::apply(root, command)?,
        EditCommand::Batch { commands } => {
            if commands.is_empty() || commands.len() > 256 {
                return Err("一次批量编辑需要 1–256 项操作".into());
            }
            if commands
                .iter()
                .any(|c| matches!(c, EditCommand::Batch { .. }))
            {
                return Err("批量编辑不能嵌套".into());
            }
            // Document::edit validates and installs only the final cloned document.
            for command in commands {
                apply(root, command)?;
            }
        }
        EditCommand::SetInfo { name, description } => {
            root["project"]["name"] = name.into();
            root["project"]["description"] = description.into();
        }
        EditCommand::AddFixture {
            name,
            profile_id,
            domain_id,
            universe,
            address,
        } => {
            let fixture_id = id();
            list(root, "fixtures")?.push(
                json!({"id":fixture_id,"name":name,"profileId":profile_id,"domainId":domain_id}),
            );
            list(root,"patches")?.push(json!({"fixtureId":fixture_id,"domainId":domain_id,"universe":universe,"address":address}));
        }
        EditCommand::UpdateFixture {
            id,
            name,
            universe,
            address,
        } => {
            let fixture = find(list(root, "fixtures")?, &id)?;
            fixture["name"] = name.into();
            let domain = fixture["domainId"].clone();
            let patches = list(root, "patches")?;
            let next =
                json!({"fixtureId":id,"domainId":domain,"universe":universe,"address":address});
            if let Some(patch) = patches.iter_mut().find(|p| p["fixtureId"] == id) {
                *patch = next;
            } else {
                patches.push(next);
            }
        }
        EditCommand::RemoveFixture { id } => {
            // Reference validation rejects deletion while any group, preset or scene uses it.
            remove(list(root, "fixtures")?, &id)?;
            list(root, "patches")?.retain(|p| p["fixtureId"] != id);
        }
        EditCommand::AddScene { name } => add_scene(root, &name)?,
        EditCommand::DuplicateScene { id: source, name } => {
            let mut copy = find(list(root, "scenes")?, &source)?.clone();
            copy["id"] = id().into();
            copy["name"] = name.into();
            list(root, "scenes")?.push(copy);
        }
        EditCommand::RenameScene { id, name } => {
            find(list(root, "scenes")?, &id)?["name"] = name.into();
        }
        EditCommand::RemoveScene { id } => remove(list(root, "scenes")?, &id)?,
        EditCommand::SetSceneValue {
            scene_id,
            fixture_id,
            attribute,
            mode,
            value,
        } => {
            validate_target(root, &fixture_id, &attribute)?;
            let scene = find(list(root, "scenes")?, &scene_id)?;
            let entries = scene["assignments"].as_array_mut().ok_or("场景属性无效")?;
            let target = json!({"fixtureId":fixture_id,"attribute":attribute});
            let position = entries.iter().position(|entry| entry["target"] == target);
            if matches!(mode, ValueMode::Remove) {
                if let Some(index) = position {
                    entries.remove(index);
                }
            } else {
                let next = if matches!(mode, ValueMode::Release) {
                    json!({"target":target,"operation":"release"})
                } else {
                    json!({"target":target,"operation":"set","source":{"kind":"literal","value":{"kind":"normalized","value":value}}})
                };
                if let Some(index) = position {
                    entries[index] = next;
                } else {
                    entries.push(next);
                }
            }
        }
    }
    Ok(())
}
pub(super) fn validate_target(
    root: &Value,
    fixture_id: &str,
    attribute: &str,
) -> Result<(), String> {
    let fixture = array(&root["lighting"], "fixtures")
        .iter()
        .find(|f| f["id"] == fixture_id)
        .ok_or("灯具已不存在，请重新选择")?;
    let profile = array(&root["lighting"], "profiles")
        .iter()
        .find(|p| p["id"] == fixture["profileId"])
        .ok_or("灯具档案不存在")?;
    if !array(profile, "attributes")
        .iter()
        .any(|a| a["key"] == attribute)
    {
        return Err("灯具没有这个属性".into());
    }
    Ok(())
}

pub(super) fn list<'a>(root: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, String> {
    root["lighting"][key]
        .as_array_mut()
        .ok_or_else(|| "工程没有灯光模块".into())
}
pub(super) fn find<'a>(items: &'a mut [Value], id: &str) -> Result<&'a mut Value, String> {
    items
        .iter_mut()
        .find(|item| item["id"] == id)
        .ok_or_else(|| "对象已不存在，请重新选择".into())
}
pub(super) fn remove(items: &mut Vec<Value>, id: &str) -> Result<(), String> {
    let index = items
        .iter()
        .position(|item| item["id"] == id)
        .ok_or("对象已不存在")?;
    items.remove(index);
    Ok(())
}
fn add_scene(root: &mut Value, name: &str) -> Result<(), String> {
    let lighting = &root["lighting"];
    if array(lighting, "fixtures").is_empty() {
        return Err("请先添加灯具，再创建场景".into());
    }
    let mut assignments = Vec::new();
    for fixture in array(lighting, "fixtures") {
        let profile = array(lighting, "profiles")
            .iter()
            .find(|p| p["id"] == fixture["profileId"])
            .ok_or("灯具档案不存在")?;
        for attribute in array(profile, "attributes") {
            assignments.push(json!({"target":{"fixtureId":text(fixture,"id"),"attribute":text(attribute,"key")},"operation":"set","source":{"kind":"literal","value":attribute["default"]}}));
        }
    }
    list(root, "scenes")?.push(json!({"id":id(),"name":name,"assignments":assignments}));
    Ok(())
}
