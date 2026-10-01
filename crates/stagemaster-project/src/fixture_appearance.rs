//! Display annotations for observed wheel slots, independent of DMX and physical optics.
use crate::{FunctionDefinition, FunctionMode, ProfileDefinition, array};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const CAPABILITY: &str = "lighting.fixture-wheel-appearance";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum WheelAppearance {
    Open {},
    Color { colors: Vec<String> },
}
impl WheelAppearance {
    pub(super) fn validate(&self) -> Result<(), String> {
        if let Self::Color { colors } = self
            && (!(1..=2).contains(&colors.len())
                || colors.iter().any(|c| {
                    c.len() != 7
                        || !c.starts_with('#')
                        || !c.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
                }))
        {
            return Err("色盘外观需要 1–2 个 #RRGGBB 颜色值".into());
        }
        Ok(())
    }
}

pub(super) fn validate_channel(
    attribute: &str,
    functions: &[FunctionDefinition],
) -> Result<(), String> {
    for function in functions {
        if let Some(appearance) = &function.appearance {
            if attribute != "color-wheel" || function.mode != FunctionMode::Slot {
                return Err(format!("“{}”的外观只能用于色盘固定档位", function.name));
            }
            appearance.validate()?;
        }
    }
    Ok(())
}

pub(super) fn require(root: &mut Value, definition: &ProfileDefinition) {
    if definition.channels.iter().any(|c| {
        c.functions
            .as_ref()
            .is_some_and(|fs| fs.iter().any(|f| f.appearance.is_some()))
    }) && !array(root, "requires")
        .iter()
        .any(|r| r["key"] == CAPABILITY)
    {
        root["requires"]
            .as_array_mut()
            .expect("validated requires")
            .push(json!({"key":CAPABILITY,"version":1}));
    }
}

pub(super) fn validate(root: &Value) -> Result<(), String> {
    let present = array(&root["lighting"], "profiles").iter().any(|p| {
        array(p, "channels").iter().any(|c| {
            array(c, "functions")
                .iter()
                .any(|f| f.get("appearance").is_some())
        })
    });
    if present
        && !array(root, "requires")
            .iter()
            .any(|r| r["key"] == CAPABILITY && r["version"] == 1)
    {
        return Err("工程缺少色盘外观能力声明".into());
    }
    Ok(())
}

/// Equivalent control identities and byte mappings; display metadata and row order may differ.
pub(super) fn same_mapping(a: &[FunctionDefinition], b: &[FunctionDefinition]) -> bool {
    a.len() == b.len()
        && a.iter().all(|f| {
            b.iter().any(|g| {
                f.key == g.key
                    && f.mode == g.mode
                    && f.dmx_from == g.dmx_from
                    && f.dmx_to == g.dmx_to
                    && f.dmx_default == g.dmx_default
            })
        })
}
