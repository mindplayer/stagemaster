//! Validated fixture bindings and output encoding, independent of scene timing.
mod live;
mod manual;
use crate::{array, text};
use serde::Serialize;
use stagemaster_dmx::{
    ChannelMapping, DmxAddress, FixtureProfile, Patch, PatchedFixture, Universe,
};
use stagemaster_domain::{Attribute, AttributeAddress, FixtureId, MixMode, NormalizedValue};
use stagemaster_engine::{OutputSnapshot, ResolvedAttribute};
type Targets = Vec<(String, String)>;

pub struct CompiledOutput {
    patch: Patch,
    addresses: Vec<AttributeAddress>,
    bindings: Vec<FixtureBinding>,
    universe: u16,
    intensity: Vec<bool>,
}
struct FixtureBinding {
    id: String,
    name: String,
    address: u16,
    attributes: Vec<AttributeBinding>,
}
struct AttributeBinding {
    key: String,
    index: usize,
    functions: Option<Vec<crate::FunctionDefinition>>,
    fine: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<crate::FunctionOutput>,
}
impl CompiledOutput {
    pub(super) fn attribute_bindings(&self) -> impl Iterator<Item = (&str, &str, usize, bool)> {
        self.bindings.iter().flat_map(|fixture| {
            fixture.attributes.iter().map(move |attribute| {
                (
                    fixture.id.as_str(),
                    attribute.key.as_str(),
                    attribute.index,
                    attribute.functions.is_some(),
                )
            })
        })
    }
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
        self.render_with_master(values, stagemaster_playback::OutputMaster::default())
    }
    /// Apply numeric intensity control before encoding; stored player values remain unchanged.
    /// # Errors
    /// Rejects a value buffer from a different plan shape.
    pub fn render_with_master(
        &self,
        values: &[u16],
        master: stagemaster_playback::OutputMaster,
    ) -> Result<PreviewOutput, String> {
        if values.len() != self.addresses.len() {
            return Err("预览输出与计划不一致".into());
        }
        let value_at = |index: usize| {
            if self.intensity[index] {
                master.scale(values[index])
            } else {
                values[index]
            }
        };
        let snapshot: OutputSnapshot = self
            .addresses
            .iter()
            .enumerate()
            .map(|(index, &address)| {
                let value = value_at(index);
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
                        .map(|a| AttributeOutput {
                            key: a.key.clone(),
                            value: value_at(a.index),
                            function: a.functions.as_ref().and_then(|functions| {
                                crate::function_output::describe(
                                    functions,
                                    a.fine,
                                    value_at(a.index),
                                )
                            }),
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}
pub(super) fn compile_output(
    root: &serde_json::Value,
) -> Result<(CompiledOutput, Vec<u16>, Targets), String> {
    let lighting = &root["lighting"];
    let mut line: Option<(&str, u16)> = None;
    let mut fixtures = Vec::new();
    let mut defaults = Vec::new();
    let mut addresses = Vec::new();
    let mut targets = Vec::new();
    let mut bindings = Vec::new();
    let mut intensity = Vec::new();
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
        let intensity_keys = crate::output_intensity::keys(profile);
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
            let offsets = channel_offsets(channel)?;
            let default = crate::fixture_value::encode(profile, name, &attribute["default"])?;
            attributes.push(AttributeBinding {
                key: name.into(),
                index: defaults.len(),
                functions: crate::fixture_value::functions(channel)?,
                fine: offsets.len() == 2,
            });
            intensity.push(intensity_keys.contains(&name));
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
            intensity,
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

fn channel_offsets(channel: &serde_json::Value) -> Result<Vec<u16>, String> {
    array(channel, "offsets")
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|n| u16::try_from(n).ok())
                .ok_or_else(|| "通道偏移无效".to_string())
        })
        .collect()
}
