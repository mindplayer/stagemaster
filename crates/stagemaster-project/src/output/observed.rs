use super::{AttributeOutput, CompiledOutput, FixtureOutput, PreviewOutput};

/// Read-only projection of a complete encoded output against one prepared fixture mapping.
pub struct OutputObserver {
    output: CompiledOutput,
    mapping: stagemaster_package::Output,
}
impl CompiledOutput {
    /// Prepare a decoder outside the observation loop. Never executes a program or owns an output.
    /// # Errors
    /// Rejects inconsistent internal fixture mappings.
    pub fn observer(self) -> Result<OutputObserver, String> {
        Ok(OutputObserver {
            mapping: self.portable_output()?,
            output: self,
        })
    }
}
impl OutputObserver {
    /// Restore actual emitted resolution; 8-bit channels map to 0, 257, ... 65535.
    /// # Errors
    /// Rejects a different universe. The caller must also verify the immutable project identity.
    pub fn observe(&self, universe: u16, slots: &[u8; 512]) -> Result<PreviewOutput, String> {
        if universe != self.mapping.universe {
            return Err("观察线路与固定工程配适不一致".into());
        }
        let values: Vec<_> = self
            .mapping
            .mappings
            .iter()
            .map(|m| {
                let coarse = u16::from(slots[usize::from(m.coarse - 1)]);
                m.fine.map_or(coarse * 257, |fine| {
                    (coarse << 8) | u16::from(slots[usize::from(fine - 1)])
                })
            })
            .collect();
        Ok(PreviewOutput {
            universe,
            slots: slots.to_vec(),
            fixtures: self
                .output
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
                            value: values[a.index],
                            function: a.functions.as_ref().and_then(|functions| {
                                crate::function_output::describe(functions, a.fine, values[a.index])
                            }),
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}
