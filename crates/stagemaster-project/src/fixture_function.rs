//! Typed function selections and native DMX ranges. No UI, I/O or runtime clock.
use serde::{Deserialize, Serialize};

pub const MAX_CHANNEL_FUNCTIONS: usize = 64;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FunctionMode {
    Slot,
    Range,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionDefinition {
    pub key: String,
    pub name: String,
    pub mode: FunctionMode,
    pub dmx_from: u16,
    pub dmx_to: u16,
    pub dmx_default: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionSelection {
    pub function_key: String,
    pub position: u16,
}

/// The authoring API preserves old numeric defaults and uses a typed selection for functions.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProfileDefault {
    Normalized(u16),
    Function(FunctionSelection),
}
impl From<u16> for ProfileDefault {
    fn from(value: u16) -> Self {
        Self::Normalized(value)
    }
}

/// A borrowed, validated table. Building it never changes order or allocates a duplicate table.
#[derive(Debug)]
pub struct FunctionTable<'a> {
    functions: &'a [FunctionDefinition],
    fine: bool,
}
impl<'a> FunctionTable<'a> {
    /// Validate native channel precision, non-overlapping ranges and stable semantic keys.
    /// Gaps are allowed; no selection can address a gap.
    /// # Errors
    /// Rejects empty/excessive tables, invalid keys/names, overlap and out-of-range defaults.
    pub fn new(functions: &'a [FunctionDefinition], fine: bool) -> Result<Self, String> {
        if !(1..=MAX_CHANNEL_FUNCTIONS).contains(&functions.len()) {
            return Err("每个功能通道需要 1–64 个功能区间".into());
        }
        let maximum = if fine { u16::MAX } else { 255 };
        for (index, function) in functions.iter().enumerate() {
            if !valid_key(&function.key) {
                return Err(format!("第 {} 个功能的标识无效", index + 1));
            }
            if function.name.trim().is_empty()
                || function.name.chars().count() > 256
                || function.name.chars().any(char::is_control)
            {
                return Err(format!(
                    "第 {} 个功能需要填写 1–256 个有效名称字符",
                    index + 1
                ));
            }
            if function.dmx_from > function.dmx_to
                || function.dmx_to > maximum
                || !(function.dmx_from..=function.dmx_to).contains(&function.dmx_default)
                || (function.mode == FunctionMode::Range && function.dmx_from == function.dmx_to)
            {
                return Err(format!(
                    "功能“{}”的区间或代表值无效：应在 0–{maximum} 内，连续区间起点须小于终点",
                    function.name
                ));
            }
            for previous in &functions[..index] {
                if previous.key == function.key {
                    return Err(format!(
                        "功能“{}”与“{}”的标识重复",
                        previous.name, function.name
                    ));
                }
                if function.dmx_from <= previous.dmx_to && function.dmx_to >= previous.dmx_from {
                    return Err(format!(
                        "功能“{}”与“{}”的通道区间重叠",
                        previous.name, function.name
                    ));
                }
            }
        }
        Ok(Self { functions, fine })
    }

    /// Encode a semantic choice to the existing u16 output representation.
    /// An 8-bit native value expands to both bytes, preserving exact high-byte encoding.
    /// # Errors
    /// Unknown functions or nonzero positions for discrete slots are rejected.
    pub fn encode(&self, selection: &FunctionSelection) -> Result<u16, String> {
        let function = self.function(&selection.function_key)?;
        let native = match function.mode {
            FunctionMode::Slot => {
                if selection.position != 0 {
                    return Err(format!(
                        "功能“{}”是离散档位，不能设置区间位置",
                        function.name
                    ));
                }
                function.dmx_default
            }
            FunctionMode::Range => {
                let span = u32::from(function.dmx_to - function.dmx_from);
                let offset = (span * u32::from(selection.position) + 32767) / 65535;
                function.dmx_from + u16::try_from(offset).map_err(|_| "功能输出超出数值范围")?
            }
        };
        Ok(if self.fine { native } else { native * 257 })
    }

    /// Resolve a new function's representative value without persisting an absolute DMX value.
    /// # Errors
    /// Rejects unknown keys rather than falling back to the first function.
    pub fn initial_selection(&self, key: &str) -> Result<FunctionSelection, String> {
        let function = self.function(key)?;
        let position = if function.mode == FunctionMode::Slot {
            0
        } else {
            let span = u32::from(function.dmx_to - function.dmx_from);
            let value = u32::from(function.dmx_default - function.dmx_from);
            u16::try_from((value * 65535 + span / 2) / span).map_err(|_| "功能初值超出区间范围")?
        };
        Ok(FunctionSelection {
            function_key: key.into(),
            position,
        })
    }

    fn function(&self, key: &str) -> Result<&FunctionDefinition, String> {
        self.functions
            .iter()
            .find(|function| function.key == key)
            .ok_or_else(|| format!("此灯具没有功能“{key}”"))
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key.as_bytes()[0].is_ascii_lowercase()
        && key.split(['.', '_', '-']).all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}
