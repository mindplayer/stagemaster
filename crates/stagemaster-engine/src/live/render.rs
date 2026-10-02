use super::{Error, Handle, LiveMixer};
use stagemaster_domain::{MixMode, NormalizedValue};

impl LiveMixer {
    /// Resolve semantic u16 values before any DMX encoding. No sorting or allocation.
    /// Winners identify the selected value, not a historical full contribution trace.
    /// # Errors
    /// Reject output shape atomically before writing either buffer.
    pub fn render(&self, values: &mut [u16], winners: &mut [Option<Handle>]) -> Result<(), Error> {
        if values.len() != self.layout.attributes.len() || winners.len() != values.len() {
            return Err(Error::Shape);
        }
        for (index, attribute) in self.layout.attributes.iter().enumerate() {
            let mut selected: Option<(i16, u16, u64, Handle)> = None;
            for slot in &self.slots {
                let Some(state) = slot.state else { continue };
                let cell = slot.cells[index];
                let Some(mut value) = cell.value else {
                    continue;
                };
                if attribute.intensity {
                    value = NormalizedValue::from_raw(value)
                        .scale(NormalizedValue::from_raw(state.level))
                        .raw();
                }
                let wins = selected.is_none_or(|(priority, previous, order, _)| {
                    state.priority > priority
                        || (state.priority == priority
                            && match attribute.mix {
                                MixMode::HighestTakesPrecedence => {
                                    (value, cell.order) > (previous, order)
                                }
                                MixMode::LatestTakesPrecedence => cell.order > order,
                            })
                });
                if wins {
                    selected = Some((state.priority, value, cell.order, state.handle));
                }
            }
            values[index] = selected.map_or(attribute.default, |(_, value, _, _)| value);
            winners[index] = selected.map(|(_, _, _, handle)| handle);
        }
        Ok(())
    }
}
