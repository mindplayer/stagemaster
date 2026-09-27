//! Converts a validated editing snapshot to a bounded plan and a separate output encoder.
use crate::{Document, ProjectView, array, text};
use serde::Serialize;
use stagemaster_dmx::{
    ChannelMapping, DmxAddress, FixtureProfile, Patch, PatchedFixture, Universe,
};
use stagemaster_domain::{Attribute, AttributeAddress, FixtureId, MixMode, NormalizedValue};
use stagemaster_engine::{OutputSnapshot, ResolvedAttribute};
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
    pub id: String,
    pub name: String,
    pub number: String,
}
pub struct CompiledOutput {
    patch: Patch,
    addresses: Vec<AttributeAddress>,
    bindings: Vec<FixtureBinding>,
    universe: u16,
}
struct FixtureBinding {
    id: String,
    name: String,
    address: u16,
    attributes: Vec<(String, usize)>,
}
#[derive(Serialize)]
pub struct PreviewOutput {
    pub universe: u16,
    pub slots: Vec<u8>,
    pub fixtures: Vec<FixtureOutput>,
}
#[derive(Serialize)]
pub struct FixtureOutput {
    pub id: String,
    pub name: String,
    pub address: u16,
    pub attributes: Vec<AttributeOutput>,
}
#[derive(Serialize)]
pub struct AttributeOutput {
    pub key: String,
    pub value: u16,
}
impl CompiledOutput {
    /// Lower the validated patch in the exact playback attribute order.
    /// # Errors
    /// Rejects an internally inconsistent compilation result.
    pub fn portable_output(&self) -> Result<stagemaster_package::Output, String> {
        let mappings = self
            .addresses
            .iter()
            .map(|address| {
                let fixture = self
                    .patch
                    .fixtures()
                    .iter()
                    .find(|f| f.id == address.fixture)
                    .ok_or("输出缺少灯具映射")?;
                let channel = fixture
                    .profile
                    .channels
                    .iter()
                    .find(|c| c.attribute == address.attribute)
                    .ok_or("输出缺少属性映射")?;
                Ok(stagemaster_package::Mapping {
                    coarse: fixture.address.number() + channel.coarse_offset,
                    fine: channel
                        .fine_offset
                        .map(|offset| fixture.address.number() + offset),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(stagemaster_package::Output {
            universe: self.universe,
            mappings,
        })
    }
    /// Encode complete DMX slots outside the time-critical execution module.
    /// # Errors
    /// Rejects a value buffer from a different plan shape.
    pub fn render(&self, values: &[u16]) -> Result<PreviewOutput, String> {
        if values.len() != self.addresses.len() {
            return Err("预览输出与计划不一致".into());
        }
        let snapshot: OutputSnapshot = self
            .addresses
            .iter()
            .zip(values)
            .map(|(&address, &value)| {
                (
                    address,
                    ResolvedAttribute {
                        value: NormalizedValue::from_raw(value),
                        trace: Vec::new(),
                    },
                )
            })
            .collect();
        let frames = self.patch.encode(&snapshot);
        let slots = frames
            .values()
            .next()
            .ok_or("预览没有输出线路")?
            .slots()
            .to_vec();
        Ok(PreviewOutput {
            universe: self.universe,
            slots,
            fixtures: self
                .bindings
                .iter()
                .map(|f| FixtureOutput {
                    id: f.id.clone(),
                    name: f.name.clone(),
                    address: f.address,
                    attributes: f
                        .attributes
                        .iter()
                        .map(|(key, index)| AttributeOutput {
                            key: key.clone(),
                            value: values[*index],
                        })
                        .collect(),
                })
                .collect(),
        })
    }
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
            let channels = crate::effects::compile(&scene.effects, &targets)?;
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
            plan: Plan::with_effects(defaults, steps, sequence["repeat"] == "loop", effects)?,
            output,
            id: text(sequence, "id").into(),
            name: text(sequence, "name").into(),
            revision_id: text(&root["project"], "revisionId").into(),
            steps: array(sequence, "steps")
                .iter()
                .map(|s| CompiledStep {
                    id: text(s, "id").into(),
                    name: text(s, "name").into(),
                    number: text(s, "number").into(),
                })
                .collect(),
        })
    }
}
type Targets = Vec<(String, String)>;
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
fn compile_output(root: &serde_json::Value) -> Result<(CompiledOutput, Vec<u16>, Targets), String> {
    let lighting = &root["lighting"];
    let mut line: Option<(&str, u16)> = None;
    let mut fixtures = Vec::new();
    let mut defaults = Vec::new();
    let mut addresses = Vec::new();
    let mut targets = Vec::new();
    let mut bindings = Vec::new();
    for (index, fixture) in array(lighting, "fixtures").iter().enumerate() {
        let patch = array(lighting, "patches")
            .iter()
            .find(|p| p["fixtureId"] == fixture["id"])
            .ok_or_else(|| format!("灯具“{}”尚未配适", text(fixture, "name")))?;
        let universe = small(patch, "universe")?;
        let key = (text(patch, "domainId"), universe);
        if line.is_some_and(|line| line != key) {
            return Err("当前离线预览支持一个输出域的一条线路，请将灯具配适到同一线路".into());
        }
        line = Some(key);
        let profile = array(lighting, "profiles")
            .iter()
            .find(|p| p["id"] == fixture["profileId"])
            .ok_or("灯具档案不存在")?;
        let fixture_id = FixtureId(u64::try_from(index).map_err(|_| "灯具数量超限")?);
        let mut attributes = Vec::new();
        let mut channels = Vec::new();
        for (attribute_index, attribute) in array(profile, "attributes").iter().enumerate() {
            if defaults.len() >= stagemaster_playback::MAX_ATTRIBUTES {
                return Err("当前预览最多支持 512 个灯具属性".into());
            }
            let name = text(attribute, "key");
            let attr =
                Attribute::Custom(u16::try_from(attribute_index).map_err(|_| "属性数量超限")?);
            let channel = array(profile, "channels")
                .iter()
                .find(|c| c["attribute"] == name)
                .ok_or("属性没有通道映射")?;
            let offsets = array(channel, "offsets")
                .iter()
                .map(|v| {
                    v.as_u64()
                        .and_then(|n| u16::try_from(n).ok())
                        .ok_or("通道偏移无效")
                })
                .collect::<Result<Vec<_>, _>>()?;
            let default = small(&attribute["default"], "value")?;
            attributes.push((name.into(), defaults.len()));
            defaults.push(default);
            targets.push((text(fixture, "id").into(), name.into()));
            addresses.push(AttributeAddress::new(fixture_id, attr));
            channels.push(ChannelMapping {
                attribute: attr,
                coarse_offset: offsets[0],
                fine_offset: offsets.get(1).copied(),
                default: NormalizedValue::from_raw(default),
                mix_mode: if attribute["mix"] == "htp" {
                    MixMode::HighestTakesPrecedence
                } else {
                    MixMode::LatestTakesPrecedence
                },
            });
        }
        let address = small(patch, "address")?;
        bindings.push(FixtureBinding {
            id: text(fixture, "id").into(),
            name: text(fixture, "name").into(),
            address,
            attributes,
        });
        fixtures.push(PatchedFixture {
            id: fixture_id,
            name: text(fixture, "name").into(),
            universe: Universe::new(universe).map_err(|e| e.to_string())?,
            address: DmxAddress::new(address).map_err(|e| e.to_string())?,
            profile: FixtureProfile {
                manufacturer: text(profile, "manufacturer").into(),
                model: text(profile, "model").into(),
                mode: text(profile, "mode").into(),
                footprint: small(profile, "footprint")?,
                channels,
            },
        });
    }
    let universe = line.ok_or("请先配适灯具")?.1;
    Ok((
        CompiledOutput {
            patch: Patch::new(fixtures).map_err(|e| e.to_string())?,
            addresses,
            bindings,
            universe,
        },
        defaults,
        targets,
    ))
}
fn small(value: &serde_json::Value, key: &str) -> Result<u16, String> {
    value[key]
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .ok_or_else(|| format!("{key} 数值越界"))
}
