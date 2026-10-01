//! Explicit exchange compatibility for stable control-slot identities, not color matching.
use crate::{FunctionDefinition, FunctionMode};

pub(super) fn compatible(
    source: &[FunctionDefinition],
    target: &[FunctionDefinition],
    allow_color_slots: bool,
) -> bool {
    source.len() == target.len()
        && source.iter().all(|f| {
            target.iter().any(|g| {
                f.key == g.key
                    && f.mode == g.mode
                    && ((allow_color_slots && f.mode == FunctionMode::Slot)
                        || (f.dmx_from == g.dmx_from
                            && f.dmx_to == g.dmx_to
                            && f.dmx_default == g.dmx_default))
            })
        })
}
