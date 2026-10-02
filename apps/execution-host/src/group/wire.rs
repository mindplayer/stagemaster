use crate::wire::Failure;
use serde::Deserialize;
use stagemaster_project::ProfileDefault;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Operation {
    Start { step: String },
    Pause {},
    Resume {},
    Next {},
    Stop {},
    Level { value: u16 },
    Patch { changes: Vec<Edit> },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Edit {
    pub fixture_id: String,
    pub attribute: String,
    pub value: Value,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Value {
    Release {},
    Normalized { value: u16 },
    Function { function_key: String, position: u16 },
}
impl Value {
    pub fn semantic(&self) -> Option<ProfileDefault> {
        match self {
            Self::Release {} => None,
            Self::Normalized { value } => Some(ProfileDefault::Normalized(*value)),
            Self::Function {
                function_key,
                position,
            } => Some(ProfileDefault::Function(
                stagemaster_project::FunctionSelection {
                    function_key: function_key.clone(),
                    position: *position,
                },
            )),
        }
    }
}
pub(crate) fn identity(value: &str) -> Result<Uuid, Failure> {
    let id = Uuid::parse_str(value).map_err(|_| Failure::invalid())?;
    if id.is_nil() || id.to_string() != value {
        return Err(Failure::invalid());
    }
    Ok(id)
}
impl Operation {
    pub fn validate(&self) -> Result<(), Failure> {
        match self {
            Self::Start { step } => {
                identity(step)?;
            }
            Self::Patch { changes } => {
                if changes.is_empty() || changes.len() > 512 {
                    return Err(Failure::invalid());
                }
                for edit in changes {
                    identity(&edit.fixture_id)?;
                    if edit.attribute.is_empty() || edit.attribute.len() > 128 {
                        return Err(Failure::invalid());
                    }
                    if let Value::Function { function_key, .. } = &edit.value
                        && (function_key.is_empty() || function_key.len() > 128)
                    {
                        return Err(Failure::invalid());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
