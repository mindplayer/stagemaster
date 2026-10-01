use super::{
    EffectTemplateDefinition, EffectTemplateFile, EffectTemplateRecipe, EffectTemplateTiming,
    IntensityWaveform,
};
use crate::{
    Document, EditCommand, EffectEdit, EffectValues, PlanUsage, SceneEffect, Waveform, array, text,
};
use serde::Serialize;
use std::collections::BTreeSet;

/// Opaque reviewed candidate, held by the application adapter. Never deserialized from UI.
pub struct EffectTemplateReview {
    base: Document,
    candidate: Document,
    view: EffectTemplateReviewView,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectTemplateReviewView {
    pub scene_id: String,
    pub effect: SceneEffect,
    pub usage: PlanUsage,
}
impl EffectTemplateReview {
    #[must_use]
    pub fn view(&self) -> &EffectTemplateReviewView {
        &self.view
    }
}
impl Document {
    /// Export only a supported brightness recipe; no fixture identity, address or base color.
    /// # Errors
    /// Rejects missing effects or effects this template format cannot fully represent.
    pub fn effect_template_file(
        &self,
        scene_id: &str,
        effect_id: &str,
    ) -> Result<EffectTemplateFile, String> {
        let scene = array(&self.root["lighting"], "scenes")
            .iter()
            .find(|s| s["id"] == scene_id)
            .ok_or("场景不存在")?;
        let effect = crate::effects::read(scene)
            .into_iter()
            .find(|e| e.id == effect_id)
            .ok_or("效果不存在")?;
        let waveform = match effect.waveform {
            Waveform::Smooth => IntensityWaveform::Smooth,
            Waveform::Triangle => IntensityWaveform::Triangle,
            Waveform::Pulse => IntensityWaveform::Pulse,
            _ => return Err("当前模板文件只支持亮度呼吸、往返和脉冲".into()),
        };
        let [
            EffectValues::Range {
                attribute,
                low,
                high,
            },
        ] = effect.channels.as_slice()
        else {
            return Err("请只导出单一亮度属性的效果".into());
        };
        if attribute != "dimmer" || effect.target_path.is_some() {
            return Err("当前模板文件只支持亮度效果".into());
        }
        EffectTemplateFile::create(EffectTemplateDefinition {
            name: effect.name,
            recipe: EffectTemplateRecipe::IntensityWave {
                waveform,
                low: *low,
                high: *high,
                duty_percent: effect.duty_percent,
            },
            timing: EffectTemplateTiming {
                period_ms: effect.period_ms,
                phase_degrees: effect.phase_degrees,
                spread_degrees: effect.spread_degrees,
                reverse_order: effect.reverse,
            },
        })
    }
    /// Bind and compile an immutable template against an exact desktop project snapshot.
    /// # Errors
    /// Rejects incompatible/duplicate targets, attribute conflicts and compile budgets.
    pub fn review_effect_template(
        &self,
        file: &EffectTemplateFile,
        scene_id: &str,
        fixture_ids: &[String],
    ) -> Result<EffectTemplateReview, String> {
        if fixture_ids.is_empty() || fixture_ids.len() > 512 {
            return Err("请选择 1–512 台灯具".into());
        }
        let mut unique = BTreeSet::new();
        for fixture_id in fixture_ids {
            if !unique.insert(fixture_id) {
                return Err("模板灯序包含重复灯具".into());
            }
            let fixture = array(&self.root["lighting"], "fixtures")
                .iter()
                .find(|f| f["id"] == *fixture_id)
                .ok_or("所选灯具不存在")?;
            crate::editing::validate_target(&self.root, fixture_id, "dimmer")
                .map_err(|e| format!("灯具“{}”不能使用亮度模板：{e}", text(fixture, "name")))?;
        }
        let definition = &file.template().definition;
        let EffectTemplateRecipe::IntensityWave {
            waveform,
            low,
            high,
            duty_percent,
        } = definition.recipe;
        let timing = &definition.timing;
        let effect = SceneEffect {
            id: crate::id(),
            name: definition.name.clone(),
            enabled: true,
            fixture_ids: fixture_ids.to_vec(),
            period_ms: timing.period_ms,
            phase_degrees: timing.phase_degrees,
            spread_degrees: timing.spread_degrees,
            reverse: timing.reverse_order,
            duty_percent,
            target_path: None,
            waveform: match waveform {
                IntensityWaveform::Smooth => Waveform::Smooth,
                IntensityWaveform::Triangle => Waveform::Triangle,
                IntensityWaveform::Pulse => Waveform::Pulse,
            },
            channels: vec![EffectValues::Range {
                attribute: "dimmer".into(),
                low,
                high,
            }],
            template_source: Some(Box::new(file.source())),
        };
        let mut candidate = self.clone();
        candidate.edit(EditCommand::Effect {
            command: EffectEdit::Put {
                scene_id: scene_id.into(),
                effect: effect.clone(),
            },
        })?;
        let compiled = candidate.compile_scene(scene_id)?;
        let view = EffectTemplateReviewView {
            scene_id: scene_id.into(),
            effect,
            usage: PlanUsage::from(&compiled.plan),
        };
        Ok(EffectTemplateReview {
            base: self.clone(),
            candidate,
            view,
        })
    }
    /// Commit one already reviewed candidate. Application history owns undo/redo.
    /// # Errors
    /// Rejects any changed content, including unsaved edits with the same revision ID.
    pub fn apply_effect_template(&mut self, review: EffectTemplateReview) -> Result<(), String> {
        if *self != review.base {
            return Err("工程内容已改变，请重新检查灯效模板".into());
        }
        self.root = review.candidate.root;
        Ok(())
    }
}
