//! Performance policy for semantic choices, separate from native range math.
use crate::{FunctionDefinition, FunctionMode};

pub(super) fn allowed(attribute: &str, function: &FunctionDefinition) -> bool {
    if attribute == crate::fixture_program::KEY {
        return function.key == "external";
    }
    if function.key.split(['.', '_', '-']).any(|part| {
        [
            "sound",
            "soundcontrol",
            "soundactivated",
            "auto",
            "automatic",
            "random",
            "reset",
            "macro",
            "program",
        ]
        .contains(&part)
    }) {
        return false;
    }
    if crate::fixture_emitter_function::supported(attribute) {
        return crate::fixture_emitter_function::validate_channel(
            attribute,
            std::slice::from_ref(function),
        )
        .is_ok();
    }
    let slot = function.mode == FunctionMode::Slot;
    match attribute {
        "color-wheel" => slot,
        "gobo-wheel" => slot || function.key == "shake" || function.key.starts_with("shake-"),
        "shutter" => {
            (slot && ["open", "closed"].contains(&function.key.as_str()))
                || (!slot && (function.key == "strobe" || function.key.starts_with("strobe-")))
        }
        "prism" => slot && ["off", "on"].contains(&function.key.as_str()),
        _ => false,
    }
}

pub(super) fn validate(attribute: &str, function: &FunctionDefinition) -> Result<(), String> {
    if !allowed(attribute, function) {
        return Err(format!(
            "{}的功能“{}”已屏蔽：声控、自走、自动轮盘、复位及未知控制宏不可用于演出",
            crate::view::attribute_label(attribute),
            function.name
        ));
    }
    Ok(())
}
