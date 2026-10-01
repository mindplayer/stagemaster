use super::{EffectTemplate, EffectTemplateDefinition, EffectTemplateSource};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const MAX_EFFECT_TEMPLATE_BYTES: usize = 16 * 1024;
#[derive(Clone, Debug)]
pub struct EffectTemplateFile(pub(super) EffectTemplate);
impl EffectTemplateFile {
    /// Read a bounded, strict semantic template without project or device access.
    /// # Errors
    /// Rejects unknown versions, fields, recipes, duplicate keys and invalid values.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_EFFECT_TEMPLATE_BYTES {
            return Err("灯效模板超过 16 KiB 限制".into());
        }
        let raw =
            crate::strict_json::decode(bytes).map_err(|e| format!("灯效模板解析失败：{e}"))?;
        validate(&raw)?;
        Ok(Self(
            serde_json::from_value(raw).map_err(|_| "灯效模板字段无效")?,
        ))
    }
    /// Create a new independent template identity and revision.
    /// # Errors
    /// Rejects invalid names, recipes and timing through the same file validator.
    pub fn create(definition: EffectTemplateDefinition) -> Result<Self, String> {
        let template = EffectTemplate {
            format: "stagemaster-effect-template".into(),
            format_version: definition.recipe.format_version(),
            template_id: crate::id(),
            revision: crate::id(),
            definition,
        };
        let bytes = serde_json::to_vec(&template).map_err(|_| "灯效模板编码失败")?;
        Self::decode(&bytes)
    }
    /// # Errors
    /// Returns an encoding error or rejects an oversized formatted template.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let bytes = serde_json::to_vec_pretty(&self.0).map_err(|_| "灯效模板编码失败")?;
        if bytes.len() > MAX_EFFECT_TEMPLATE_BYTES {
            return Err("灯效模板超过 16 KiB 限制".into());
        }
        Ok(bytes)
    }
    #[must_use]
    pub fn template(&self) -> &EffectTemplate {
        &self.0
    }
    #[must_use]
    pub fn source(&self) -> EffectTemplateSource {
        EffectTemplateSource {
            template: self.0.clone(),
            sha256: digest(&self.0),
        }
    }
}
pub(super) fn validate(raw: &Value) -> Result<(), String> {
    crate::schema::effect_template()
        .validate(raw)
        .map_err(|e| format!("灯效模板字段不符合格式要求：{}", e.instance_path()))?;
    if raw["definition"]["name"]
        .as_str()
        .is_none_or(|v| v.trim().is_empty())
    {
        return Err("灯效模板名称不能为空".into());
    }
    let template: EffectTemplate =
        serde_json::from_value(raw.clone()).map_err(|_| "灯效模板字段无效")?;
    if let super::EffectTemplateRecipe::IntensityKeyframes { keyframes } =
        &template.definition.recipe
    {
        crate::effects::validate_frame_order(keyframes)?;
    }
    Ok(())
}
pub(super) fn digest(template: &EffectTemplate) -> String {
    // Value uses sorted object keys. Supported recipes contain no floats.
    let value = serde_json::to_value(template).expect("typed template");
    let bytes = serde_json::to_vec(&value).expect("JSON value");
    format!("{:x}", Sha256::digest(bytes))
}
