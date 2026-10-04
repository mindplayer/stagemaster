use super::CompiledOutput;
use crate::{FunctionTable, ProfileDefault};

impl CompiledOutput {
    /// Pair actual pre-level manual values with the same prepared target mapping.
    pub fn manual_readings<'a>(
        &'a self,
        values: &'a [Option<u16>],
    ) -> impl Iterator<Item = (&'a str, &'a str, u16)> + 'a {
        self.attribute_bindings()
            .filter_map(move |(fixture, attribute, index, _)| {
                values
                    .get(index)
                    .copied()
                    .flatten()
                    .map(|value| (fixture, attribute, value))
            })
    }
    /// Project a bounded manual ownership mask through this immutable fixture mapping.
    /// Reads only; never changes ownership or claims final mixed values.
    pub fn manual_targets<'a>(
        &'a self,
        held: &'a [u64; 8],
    ) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.attribute_bindings()
            .filter_map(move |(fixture, attribute, index, _)| {
                held.get(index / 64)
                    .is_some_and(|word| word & (1 << (index % 64)) != 0)
                    .then_some((fixture, attribute))
            })
    }

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
                    if attribute == crate::fixture_program::KEY {
                        crate::fixture_program::validate_selection(selection)?;
                    }
                    let function = functions
                        .iter()
                        .find(|f| f.key == selection.function_key)
                        .ok_or("手动功能不存在")?;
                    crate::fixture_function_safety::validate(attribute, function)?;
                    FunctionTable::new(functions, binding.fine)?.encode(selection)
                }
                _ => Err("手动属性类型不匹配，功能通道必须明确选择功能".into()),
            })
            .transpose()?;
        Ok((binding.index, value))
    }
}
