//! Prepare semantic mixing metadata in the exact existing playback/encoder order.
use super::CompiledOutput;
use stagemaster_engine::live::{Attribute, Layout, LayoutId};

impl CompiledOutput {
    /// Build immutable mixing metadata; discrete functions never use numeric attenuation.
    /// # Errors
    /// Reject unsupported discrete policies or an inconsistent prepared output.
    pub fn live_layout(&self, identity: LayoutId) -> Result<Layout, String> {
        let attributes = self
            .addresses
            .iter()
            .enumerate()
            .map(|(index, address)| {
                let channel = self
                    .patch
                    .fixtures()
                    .iter()
                    .find(|f| f.id == address.fixture)
                    .and_then(|f| {
                        f.profile
                            .channels
                            .iter()
                            .find(|c| c.attribute == address.attribute)
                    })
                    .ok_or("输出缺少混合属性映射")?;
                let binding = self
                    .bindings
                    .iter()
                    .flat_map(|f| &f.attributes)
                    .find(|a| a.index == index)
                    .ok_or("输出缺少属性来源")?;
                Ok(Attribute {
                    default: channel.default.raw(),
                    mix: channel.mix_mode,
                    intensity: self.intensity[index],
                    discrete: binding.functions.is_some(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Layout::new(identity, attributes).map_err(|e| e.to_string())
    }
}
