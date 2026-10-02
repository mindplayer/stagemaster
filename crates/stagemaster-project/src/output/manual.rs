use super::CompiledOutput;
use crate::{FunctionTable, ProfileDefault};

impl CompiledOutput {
    /// Resolve a semantic manual edit against this prepared output's fixture bindings.
    /// None releases ownership; a numeric zero still owns the attribute. No output is changed.
    /// # Errors
    /// Reject unknown targets, wrong value types and invalid function selections.
    pub fn manual_value(
        &self,
        fixture_id: &str,
        attribute: &str,
        value: Option<&ProfileDefault>,
    ) -> Result<(usize, Option<u16>), String> {
        let binding = self
            .bindings
            .iter()
            .find(|f| f.id == fixture_id)
            .and_then(|f| f.attributes.iter().find(|a| a.key == attribute))
            .ok_or("手动属性不在已准备的灯具映射中")?;
        let value = value
            .map(|value| match (&binding.functions, value) {
                (None, ProfileDefault::Normalized(value)) => Ok(*value),
                (Some(functions), ProfileDefault::Function(selection)) => {
                    FunctionTable::new(functions, binding.fine)?.encode(selection)
                }
                _ => Err("手动属性类型不匹配，功能通道必须明确选择功能".into()),
            })
            .transpose()?;
        Ok((binding.index, value))
    }
}
