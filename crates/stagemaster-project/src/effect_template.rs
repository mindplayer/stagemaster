//! Portable semantic recipes; channel allocation belongs to the project adapter.
mod binding;
mod codec;
mod recipe;
mod source;
pub use binding::{EffectTemplateReview, EffectTemplateReviewView};
pub use codec::{EffectTemplateFile, MAX_EFFECT_TEMPLATE_BYTES};
pub(super) use source::validate_project;
pub(super) const CAPABILITY: &str = "lighting.effects.template-source";
pub(super) const KEYFRAME_CAPABILITY: &str = "lighting.effects.template-keyframes";
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectTemplate {
    pub format: String,
    pub format_version: u16,
    pub template_id: String,
    pub revision: String,
    pub definition: EffectTemplateDefinition,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectTemplateDefinition {
    pub name: String,
    pub recipe: EffectTemplateRecipe,
    pub timing: EffectTemplateTiming,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EffectTemplateRecipe {
    IntensityWave {
        waveform: IntensityWaveform,
        /// Normalized intensity, not a DMX slot value.
        low: u16,
        high: u16,
        #[serde(rename = "dutyPercent")]
        duty_percent: u8,
    },
    IntensityKeyframes {
        keyframes: Vec<crate::EffectKeyframe>,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IntensityWaveform {
    Smooth,
    Triangle,
    Pulse,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectTemplateTiming {
    pub period_ms: u32,
    pub phase_degrees: u16,
    pub spread_degrees: u16,
    pub reverse_order: bool,
}
/// Immutable origin only. The applied scene effect remains independently editable.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectTemplateSource {
    pub template: EffectTemplate,
    pub sha256: String,
}
