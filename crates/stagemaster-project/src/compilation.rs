//! Converts a validated editing snapshot to a bounded plan and a separate output encoder.
use crate::output::{CompiledOutput, compile_output};
use crate::{Document, ProjectView, array, text};
use serde::Serialize;
use stagemaster_playback::{Plan, Step};

pub struct CompiledSequence {
    pub plan: Plan,
    pub output: CompiledOutput,
    pub id: String,
    pub name: String,
    pub revision_id: String,
    pub steps: Vec<CompiledStep>,
}
#[derive(Clone, Serialize)]
pub struct CompiledStep {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<crate::StepScript>,
    pub id: String,
    pub name: String,
    pub number: String,
}
impl Document {
    /// Compile a single, self-contained lighting list without changing the document.
    /// # Errors
    /// Refuses unsupported output layouts, unpatched fixtures and plan budget violations.
    pub fn compile_sequence(&self, sequence_id: &str) -> Result<CompiledSequence, String> {
        self.compile_sequence_view(sequence_id, &self.view())
    }
    // The check service supplies this document's own view once for the whole report.
    pub(super) fn compile_sequence_view(
        &self,
        sequence_id: &str,
        view: &ProjectView,
    ) -> Result<CompiledSequence, String> {
        let root = &self.root;
        let lighting = &root["lighting"];
        let sequence = array(lighting, "sequences")
            .iter()
            .find(|s| s["id"] == sequence_id)
            .ok_or("场景列表不存在")?;
        self.compile_steps(sequence, view)
    }
    /// Compile a scene as one held step using the same execution semantics as a list.
    /// # Errors
    /// Refuses missing scenes, unsupported patches and execution budget violations.
    pub fn compile_scene(&self, scene_id: &str) -> Result<CompiledSequence, String> {
        self.compile_scene_view(scene_id, &self.view())
    }
    pub(super) fn compile_scene_view(
        &self,
        scene_id: &str,
        view: &ProjectView,
    ) -> Result<CompiledSequence, String> {
        let scene = array(&self.root["lighting"], "scenes")
            .iter()
            .find(|scene| scene["id"] == scene_id)
            .ok_or("场景不存在")?;
        let sequence = serde_json::json!({
            "id":scene_id,"name":scene["name"],"tracking":"isolated","repeat":"once",
            "steps":[{"id":scene_id,"name":scene["name"],"number":"1","sceneId":scene_id,
            "delay":{"ticks":"0","ticksPerSecond":"1000"},"fade":{"ticks":"0","ticksPerSecond":"1000"},
            "advance":{"kind":"manual"}}]
        });
        self.compile_steps(&sequence, view)
    }
    fn compile_steps(
        &self,
        sequence: &serde_json::Value,
        view: &ProjectView,
    ) -> Result<CompiledSequence, String> {
        let root = &self.root;
        let (output, defaults, targets) = compile_output(root)?;
        check_plan_size(array(sequence, "steps").len(), defaults.len())?;
        let mut previous = defaults.clone();
        let mut steps = Vec::new();
        let mut effects = Vec::new();
        let mut effect_count = 0;
        let mut keyframe_count = 0;
        for step in array(sequence, "steps") {
            let mut target = if sequence["tracking"] == "isolated" {
                defaults.clone()
            } else {
                previous.clone()
            };
            let scene = view
                .scenes
                .iter()
                .find(|s| s.id == text(step, "sceneId"))
                .ok_or("步骤引用的场景不存在")?;
            for value in &scene.values {
                let index = targets
                    .iter()
                    .position(|(fixture, key)| {
                        fixture == &value.fixture_id && key == &value.attribute
                    })
                    .ok_or("场景属性未纳入计划")?;
                target[index] = if value.mode == "release" {
                    defaults[index]
                } else {
                    u16::try_from(value.value.ok_or("场景引用没有对应值")?)
                        .map_err(|_| "场景值越界")?
                };
            }
            previous.clone_from(&target);
            keyframe_count += crate::effects::keyframe_count(&scene.effects);
            if keyframe_count > stagemaster_playback::MAX_KEYFRAMES {
                return Err(format!(
                    "列表关键帧超出计划容量：已累计 {keyframe_count}，上限 {}；请拆分列表或减少关键帧",
                    stagemaster_playback::MAX_KEYFRAMES
                ));
            }
            let channels = crate::effect_compile::compile(root, &scene.effects, &targets, &target)
                .map_err(|reason| format!("场景“{}”：{reason}", scene.name))?;
            effect_count += channels.len();
            if effect_count > stagemaster_playback::MAX_EFFECT_CHANNELS {
                return Err(format!(
                    "列表效果通道超出计划容量：已累计 {effect_count}，上限 {}；请拆分列表或减少效果",
                    stagemaster_playback::MAX_EFFECT_CHANNELS
                ));
            }
            effects.push(channels);
            steps.push(Step {
                target,
                delay_ms: crate::sequence::duration_ms(&step["delay"])?,
                fade_ms: crate::sequence::duration_ms(&step["fade"])?,
                wait_ms: if step["advance"]["kind"] == "after" {
                    Some(crate::sequence::duration_ms(&step["advance"]["wait"])?)
                } else {
                    None
                },
            });
        }
        Ok(CompiledSequence {
            plan: Plan::with_snap_attributes(
                defaults,
                steps,
                sequence["repeat"] == "loop",
                effects,
                targets
                    .iter()
                    .enumerate()
                    .filter_map(|(index, (fixture_id, key))| {
                        let fixture = view.fixtures.iter().find(|f| f.id == *fixture_id)?;
                        fixture
                            .attributes
                            .iter()
                            .find(|a| a.key == *key)?
                            .function
                            .as_ref()?;
                        Some(u16::try_from(index).expect("bounded attribute index"))
                    })
                    .collect(),
            )?,
            output,
            id: text(sequence, "id").into(),
            name: text(sequence, "name").into(),
            revision_id: text(&root["project"], "revisionId").into(),
            steps: array(sequence, "steps")
                .iter()
                .map(|s| CompiledStep {
                    script: crate::sequence_script::read(s),
                    id: text(s, "id").into(),
                    name: text(s, "name").into(),
                    number: text(s, "number").into(),
                })
                .collect(),
        })
    }
}
fn check_plan_size(steps: usize, attributes: usize) -> Result<(), String> {
    use stagemaster_playback::{MAX_STEPS, MAX_TARGET_VALUES};
    if steps > MAX_STEPS {
        return Err(format!(
            "列表步骤超出计划容量：{steps} / {MAX_STEPS} 步；请拆分列表或删除多余步骤"
        ));
    }
    let targets = steps * attributes;
    if targets > MAX_TARGET_VALUES {
        return Err(format!(
            "列表目标数值超出计划容量：{steps} 步 × {attributes} 个属性 = {targets}，上限 {MAX_TARGET_VALUES}；请拆分列表"
        ));
    }
    Ok(())
}
