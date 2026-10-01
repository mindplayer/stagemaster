//! Explicit, lossless profile exchange; the document transaction owns rollback.
use crate::fixture::{repatch, selection, supported_keys};
use crate::{Repatch, array, editing, text};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn apply(
    root: &mut Value,
    fixture_ids: &[String],
    profile_id: &str,
    layout: Option<Repatch>,
) -> Result<(), String> {
    selection(root, fixture_ids)?;
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
    for id in fixture_ids {
        let fixture = array(&root["lighting"], "fixtures")
            .iter()
            .find(|f| f["id"] == *id)
            .ok_or("灯具不存在")?;
        let old = profiles
            .iter()
            .find(|p| p["id"] == fixture["profileId"])
            .ok_or("原模式不存在")?;
        if old.get("positioning") != target.get("positioning") {
            return Err("运动模型不同，不能保持已记录轴角；请单独建立新的灯具并重新对焦".into());
        }
        if keys(old) != target_keys {
            return Err(format!(
                "灯具“{}”与目标模式的属性不一致，不能保留全部编排",
                text(fixture, "name")
            ));
        }
        for channel in array(old, "channels") {
            let other = array(target, "channels")
                .iter()
                .find(|c| c["attribute"] == channel["attribute"])
                .ok_or("目标属性没有通道映射")?;
            if channel.get("functions") != other.get("functions") {
                return Err("功能区间定义不同，不能直接保留编排；请先建立明确的功能映射".into());
            }
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
    for id in fixture_ids {
        editing::find(editing::list(root, "fixtures")?, id)?["profileId"] = profile_id.into();
    }
    if let Some(layout) = layout {
        repatch(root, fixture_ids, &layout)?;
    }
    Ok(())
}
