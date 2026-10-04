//! Review and atomically apply sparse manual values to one explicitly chosen scene.
use crate::{Document, ManualSceneCapture, array, text};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualMergeSummary {
    pub scene_id: String,
    pub scene_name: String,
    pub added: usize,
    pub replaced: usize,
    pub unchanged: usize,
    pub preserved: usize,
    pub effects: usize,
    pub rows: Vec<ManualMergeRow>,
}
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualMergeRow {
    pub fixture_id: String,
    pub attribute: String,
    pub change: ManualMergeChange,
    pub previous_mode: String,
    pub previous_value: Option<u16>,
    pub previous_preset: Option<String>,
    pub effect_names: Vec<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManualMergeChange {
    Added,
    Replaced,
    Unchanged,
}
#[derive(Clone)]
pub struct ManualSceneMerge {
    capture: ManualSceneCapture,
    baseline: Value,
    summary: ManualMergeSummary,
}
impl ManualSceneMerge {
    #[must_use]
    pub fn summary(&self) -> &ManualMergeSummary {
        &self.summary
    }
}
impl Document {
    /// Freeze a sparse merge and its impact, without changing any project or runtime state.
    /// # Errors
    /// Reject incompatible fixtures and a missing target scene.
    pub fn prepare_manual_scene_merge(
        &self,
        capture: &ManualSceneCapture,
        scene_id: &str,
    ) -> Result<ManualSceneMerge, String> {
        capture.check(self)?;
        let baseline = scene(self, scene_id)?.clone();
        let lighting = &self.root["lighting"];
        let effects = crate::effects::read(&baseline);
        let mut summary = ManualMergeSummary {
            scene_id: scene_id.into(),
            scene_name: text(&baseline, "name").into(),
            added: 0,
            replaced: 0,
            unchanged: 0,
            preserved: array(&baseline, "assignments").len(),
            effects: effects.len(),
            rows: Vec::new(),
        };
        for incoming in capture.assignments() {
            let target = &incoming["target"];
            let fixture_id = text(target, "fixtureId");
            let attribute = text(target, "attribute");
            let old = array(&baseline, "assignments")
                .iter()
                .find(|a| a["target"] == *target);
            let change = match old {
                None => {
                    summary.added += 1;
                    ManualMergeChange::Added
                }
                Some(a) if a == incoming => {
                    summary.unchanged += 1;
                    ManualMergeChange::Unchanged
                }
                Some(_) => {
                    summary.replaced += 1;
                    ManualMergeChange::Replaced
                }
            };
            if old.is_some() {
                summary.preserved -= 1;
            }
            let previous_value = old
                .and_then(|a| crate::library::resolved_value(lighting, a))
                .map(|v| {
                    crate::fixture_value::encode(
                        crate::fixture_value::profile(lighting, fixture_id)?,
                        attribute,
                        &v,
                    )
                })
                .transpose()?;
            let previous_preset = old.and_then(|a| {
                array(lighting, "presets")
                    .iter()
                    .find(|p| p["id"] == a["source"]["presetId"])
                    .map(|p| text(p, "name").to_owned())
            });
            let previous_mode = old
                .map_or("absent", |a| {
                    if a["operation"] == "release" {
                        "release"
                    } else {
                        text(&a["source"], "kind")
                    }
                })
                .into();
            summary.rows.push(ManualMergeRow {
                fixture_id: fixture_id.into(),
                attribute: attribute.into(),
                change,
                previous_mode,
                previous_value,
                previous_preset,
                effect_names: effects
                    .iter()
                    .filter(|e| {
                        e.enabled
                            && e.fixture_ids.iter().any(|f| f == fixture_id)
                            && e.channels.iter().any(|c| c.attribute() == attribute)
                    })
                    .map(|e| e.name.clone())
                    .collect(),
            });
        }
        Ok(ManualSceneMerge {
            capture: capture.clone(),
            baseline,
            summary,
        })
    }

    /// Apply exactly the reviewed merge. Unmentioned data and shared presets remain intact.
    /// # Errors
    /// Reject changed context, validation or capacity without changing the document.
    pub fn merge_manual_scene(&mut self, merge: &ManualSceneMerge) -> Result<(), String> {
        merge.capture.check(self)?;
        if scene(self, &merge.summary.scene_id)? != &merge.baseline {
            return Err("目标场景已变化，请重新采集并审阅".into());
        }
        if self
            .prepare_manual_scene_merge(&merge.capture, &merge.summary.scene_id)?
            .summary
            != merge.summary
        {
            return Err("场景引用内容已变化，请重新采集并审阅".into());
        }
        let mut next = self.root.clone();
        let target = crate::editing::list(&mut next, "scenes")?
            .iter_mut()
            .find(|s| s["id"] == merge.summary.scene_id)
            .ok_or("目标场景已不存在")?;
        let assignments = target["assignments"].as_array_mut().ok_or("场景属性无效")?;
        for incoming in merge.capture.assignments() {
            if let Some(old) = assignments
                .iter_mut()
                .find(|a| a["target"] == incoming["target"])
            {
                *old = incoming.clone();
            } else {
                assignments.push(incoming.clone());
            }
        }
        crate::validation::validate(&next)?;
        crate::encoding::validate_capacity(&next)?;
        self.root = next;
        Ok(())
    }
}
fn scene<'a>(document: &'a Document, id: &str) -> Result<&'a Value, String> {
    array(&document.root["lighting"], "scenes")
        .iter()
        .find(|s| s["id"] == id)
        .ok_or_else(|| "目标场景已不存在".into())
}
