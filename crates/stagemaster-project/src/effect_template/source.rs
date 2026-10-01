use super::{CAPABILITY, EffectTemplateSource, codec};
use crate::array;
use serde_json::Value;
use std::collections::BTreeMap;

pub(in crate::effect_template) fn validate(source: &EffectTemplateSource) -> Result<(), String> {
    codec::validate(&serde_json::to_value(&source.template).map_err(|_| "灯效来源无效")?)?;
    if source.sha256 != codec::digest(&source.template) {
        return Err("灯效模板来源内容与摘要不一致".into());
    }
    Ok(())
}
pub(crate) fn validate_project(root: &Value) -> Result<(), String> {
    let mut identities = BTreeMap::new();
    for scene in array(&root["lighting"], "scenes") {
        for effect in array(scene, "effects") {
            if let Some(source) = effect.get("templateSource") {
                if !array(root, "requires")
                    .iter()
                    .any(|r| r["key"] == CAPABILITY)
                {
                    return Err("灯效模板来源缺少工程能力声明".into());
                }
                let source: EffectTemplateSource =
                    serde_json::from_value(source.clone()).map_err(|_| "灯效模板来源字段无效")?;
                validate(&source)?;
                let key = (&source.template.template_id, &source.template.revision);
                if let Some(previous) =
                    identities.insert((key.0.clone(), key.1.clone()), source.sha256.clone())
                    && previous != source.sha256
                {
                    return Err("相同灯效模板身份与修订对应了不同内容，请使用新的模板修订".into());
                }
            }
        }
    }
    Ok(())
}
