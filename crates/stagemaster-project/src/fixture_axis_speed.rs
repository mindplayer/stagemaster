//! A shared pan/tilt speed channel is a control position, not a clock or physical velocity.
use crate::{ProfileChannel, ProfileDefault};

pub(crate) const KEY: &str = "pan-tilt-speed";
pub(crate) const LABEL: &str = "两轴速度控制";

pub(crate) fn validate(channel: &ProfileChannel) -> Result<(), String> {
    if channel.attribute == KEY
        && (channel.functions.is_some()
            || !matches!(channel.default_value, ProfileDefault::Normalized(_)))
    {
        return Err(format!("{LABEL}仅支持全范围线性通道和连续默认值"));
    }
    Ok(())
}
