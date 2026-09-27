//! Fixture definitions and explicit exchanges. Never evaluates output or touches hardware.
use crate::{array, editing, id, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileChannel {
    pub attribute: String,
    /// User-facing, one-based physical channel numbers. Fine may precede coarse.
    pub coarse: u16,
    pub fine: Option<u16>,
    pub default_value: u16,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileDefinition {
    pub positioning: Option<crate::PositionModel>,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub mode: String,
    pub footprint: u16,
    pub channels: Vec<ProfileChannel>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Repatch {
    pub universe: u16,
    pub address: u16,
    pub gap: u16,
}
#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum FixtureEdit {
    SaveProfile {
        id: Option<String>,
        definition: Box<ProfileDefinition>,
    },
    RemoveProfile {
        id: String,
    },
    Repatch {
        fixture_ids: Vec<String>,
        layout: Repatch,
    },
    Exchange {
        fixture_ids: Vec<String>,
        profile_id: String,
        layout: Option<Repatch>,
    },
}

pub(super) fn supported_keys<'a>(keys: impl Iterator<Item = &'a str>) -> bool {
    let mut keys = keys.collect::<Vec<_>>();
    keys.sort_unstable();
    if keys.contains(&"pan") || keys.contains(&"tilt") {
        if keys.iter().filter(|&&k| k == "pan" || k == "tilt").count() != 2
            || !keys.contains(&"pan")
            || !keys.contains(&"tilt")
        {
            return false;
        }
        keys.retain(|&k| k != "pan" && k != "tilt");
    }
    keys == ["dimmer"]
        || keys == ["blue", "green", "red"]
        || keys == ["blue", "dimmer", "green", "red"]
}
fn build(def: &ProfileDefinition, profile_id: &str) -> Result<Value, String> {
    for (label, value) in [
        ("名称", &def.name),
        ("厂家", &def.manufacturer),
        ("型号", &def.model),
        ("模式", &def.mode),
    ] {
        if value.trim().is_empty() || value.chars().count() > 256 {
            return Err(format!("灯具{label}需要填写 1–256 个字符"));
        }
    }
    if !(1..=512).contains(&def.footprint) {
        return Err("模式占用应在 1–512 通道之间".into());
    }
    if !supported_keys(def.channels.iter().map(|c| c.attribute.as_str())) {
        return Err("当前模式编辑支持调光、完整 RGB 或调光加 RGB；不能重复属性".into());
    }
    let has_axes = def.channels.iter().any(|c| c.attribute == "pan");
    if has_axes != def.positioning.is_some() {
        return Err("水平／垂直通道必须配套定义两轴物理模型".into());
    }
    if let Some(m) = &def.positioning {
        m.head(None)?;
    }
    let mut occupied = BTreeSet::new();
    for channel in &def.channels {
        for number in std::iter::once(channel.coarse).chain(channel.fine) {
            if number == 0 || number > def.footprint {
                return Err(format!(
                    "{}的通道 {number} 超出模式占用 1–{}",
                    crate::view::attribute_label(&channel.attribute),
                    def.footprint
                ));
            }
            if !occupied.insert(number) {
                return Err(format!("模式通道 {number} 重复，请检查粗调和细调映射"));
            }
        }
    }
    let mut profile = json!({"id":profile_id,"revision":id(),"name":def.name.trim(),"manufacturer":def.manufacturer.trim(),"model":def.model.trim(),"mode":def.mode.trim(),"footprint":def.footprint,
        "attributes":def.channels.iter().map(|c|json!({"key":c.attribute,"valueType":{"kind":"normalized"},"default":{"kind":"normalized","value":c.default_value},"mix":if c.attribute=="dimmer" {"htp"} else {"ltp"}})).collect::<Vec<_>>(),
        "channels":def.channels.iter().map(|c|json!({"attribute":c.attribute,"encoding":if c.fine.is_some(){"u16-be"}else{"u8"},"offsets":std::iter::once(c.coarse).chain(c.fine).map(|n|n-1).collect::<Vec<_>>()})).collect::<Vec<_>>()
    });
    if let Some(m) = &def.positioning {
        profile["positioning"] = json!(m);
    }
    Ok(profile)
}
fn unused(root: &Value, profile_id: &str) -> Result<(), String> {
    let fixtures = array(&root["lighting"], "fixtures")
        .iter()
        .filter(|f| f["profileId"] == profile_id)
        .collect::<Vec<_>>();
    if let Some(fixture) = fixtures.first() {
        return Err(format!(
            "此模式正被“{}”等 {} 台灯具使用，请另存模式后显式替换",
            text(fixture, "name"),
            fixtures.len()
        ));
    }
    Ok(())
}
fn selection(root: &Value, ids: &[String]) -> Result<(), String> {
    if ids.is_empty() || ids.len() > 256 || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("请选择 1–256 台不重复的灯具".into());
    }
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
fn repatch(root: &mut Value, ids: &[String], layout: &Repatch) -> Result<(), String> {
    if layout.universe == 0 || !(1..=512).contains(&layout.address) || layout.gap > 511 {
        return Err("线路、起始地址或间隔超出范围".into());
    }
    let mut address = u64::from(layout.address);
    for id in ids {
        let fixture = array(&root["lighting"], "fixtures")
            .iter()
            .find(|f| f["id"] == *id)
            .ok_or("灯具不存在")?
            .clone();
        let width = array(&root["lighting"], "profiles")
            .iter()
            .find(|p| p["id"] == fixture["profileId"])
            .ok_or("模式不存在")?["footprint"]
            .as_u64()
            .ok_or("模式占用无效")?;
        if address + width - 1 > 512 {
            return Err(format!(
                "灯具“{}”的地址 {}–{} 超出 512 通道",
                text(&fixture, "name"),
                address,
                address + width - 1
            ));
        }
        let next = json!({"fixtureId":id,"domainId":fixture["domainId"],"universe":layout.universe,"address":address});
        let patches = editing::list(root, "patches")?;
        if let Some(patch) = patches.iter_mut().find(|p| p["fixtureId"] == *id) {
            *patch = next;
        } else {
            patches.push(next);
        }
        address += width + u64::from(layout.gap);
    }
    Ok(())
}
pub(super) fn apply(root: &mut Value, command: FixtureEdit) -> Result<(), String> {
    match command {
        FixtureEdit::SaveProfile {
            id: existing,
            definition,
        } => {
            if definition.positioning.is_some() {
                crate::position::require(root);
            }
            if let Some(existing) = existing {
                unused(root, &existing)?;
                let profile = build(&definition, &existing)?;
                *editing::find(editing::list(root, "profiles")?, &existing)? = profile;
            } else {
                editing::list(root, "profiles")?.push(build(&definition, &id())?);
            }
        }
        FixtureEdit::RemoveProfile { id } => {
            unused(root, &id)?;
            editing::remove(editing::list(root, "profiles")?, &id)?;
        }
        FixtureEdit::Repatch {
            fixture_ids,
            layout,
        } => {
            selection(root, &fixture_ids)?;
            repatch(root, &fixture_ids, &layout)?;
        }
        FixtureEdit::Exchange {
            fixture_ids,
            profile_id,
            layout,
        } => {
            selection(root, &fixture_ids)?;
            let profiles = array(&root["lighting"], "profiles");
            let target = profiles
                .iter()
                .find(|p| p["id"] == profile_id)
                .ok_or("目标模式不存在")?;
            let keys = |p: &Value| {
                array(p, "attributes")
                    .iter()
                    .map(|a| text(a, "key").to_owned())
                    .collect::<BTreeSet<_>>()
            };
            let target_keys = keys(target);
            if !supported_keys(target_keys.iter().map(String::as_str)) {
                return Err("目标模式尚不支持安全替换".into());
            }
            for id in &fixture_ids {
                let fixture = array(&root["lighting"], "fixtures")
                    .iter()
                    .find(|f| f["id"] == *id)
                    .ok_or("灯具不存在")?;
                let old = profiles
                    .iter()
                    .find(|p| p["id"] == fixture["profileId"])
                    .ok_or("原模式不存在")?;
                if old.get("positioning") != target.get("positioning") {
                    return Err(
                        "运动模型不同，不能保持已记录轴角；请单独建立新的灯具并重新对焦".into(),
                    );
                }
                if keys(old) != target_keys {
                    return Err(format!(
                        "灯具“{}”与目标模式的属性不一致，不能保留全部编排",
                        text(fixture, "name")
                    ));
                }
                // Even known attribute names must retain their normalized type and mixing meaning.
                for a in array(old, "attributes") {
                    let b = array(target, "attributes")
                        .iter()
                        .find(|b| b["key"] == a["key"])
                        .ok_or("目标属性不存在")?;
                    if a["valueType"] != b["valueType"] || a["mix"] != b["mix"] {
                        return Err("属性的数据类型或混合方式不一致，不能安全替换".into());
                    }
                }
            }
            for id in &fixture_ids {
                editing::find(editing::list(root, "fixtures")?, id)?["profileId"] =
                    profile_id.clone().into();
            }
            if let Some(layout) = layout {
                repatch(root, &fixture_ids, &layout)?;
            }
        }
    }
    Ok(())
}
