//! Human-readable function monitoring from encoded output, never a second playback evaluator.
use crate::{FunctionDefinition, FunctionMode};
use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionOutput {
    pub key: String,
    pub name: String,
    pub dmx_value: u16,
    /// Quantized position observed on the physical channel, not the original authoring value.
    pub position: Option<u16>,
}
pub(super) fn describe(
    functions: &[FunctionDefinition],
    fine: bool,
    value: u16,
) -> Option<FunctionOutput> {
    let native = if fine { value } else { value >> 8 };
    let f = functions
        .iter()
        .find(|f| (f.dmx_from..=f.dmx_to).contains(&native))?;
    let position = if f.mode == FunctionMode::Range {
        let width = u32::from(f.dmx_to - f.dmx_from);
        Some(u16::try_from((u32::from(native - f.dmx_from) * 65535 + width / 2) / width).ok()?)
    } else {
        None
    };
    Some(FunctionOutput {
        key: f.key.clone(),
        name: f.name.clone(),
        dmx_value: native,
        position,
    })
}
