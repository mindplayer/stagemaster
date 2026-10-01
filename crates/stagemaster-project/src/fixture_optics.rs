//! Whole-range lens/iris controls; values are control positions, never physical optics.
use crate::{ProfileChannel, ProfileDefault};

#[must_use]
pub fn is_continuous_optics_attribute(key: &str) -> bool {
    label(key).is_some()
}

pub(crate) fn label(key: &str) -> Option<&'static str> {
    match key {
        "zoom" => Some("变焦"),
        "focus" => Some("调焦"),
        "iris" => Some("光圈"),
        _ => None,
    }
}

pub(crate) fn validate(channel: &ProfileChannel) -> Result<(), String> {
    if let Some(label) = label(&channel.attribute)
        && (channel.functions.is_some()
            || !matches!(channel.default_value, ProfileDefault::Normalized(_)))
    {
        return Err(format!("{label}仅支持全范围线性通道和连续默认值"));
    }
    Ok(())
}
