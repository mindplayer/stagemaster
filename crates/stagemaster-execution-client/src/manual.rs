//! Semantic manual controls against the immutable execution catalog.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualFixture {
    pub id: String,
    pub name: String,
    pub profile_name: String,
    pub universe: Option<u16>,
    pub address: Option<u16>,
    pub attributes: Vec<ManualAttribute>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualAttribute {
    pub key: String,
    pub label: String,
    pub default_value: u16,
    pub function: Option<ManualFunctionTable>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManualFunctionTable {
    pub functions: Vec<ManualFunction>,
    pub default: FunctionSelection,
    pub fine: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSelection {
    pub function_key: String,
    pub position: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualFunction {
    pub key: String,
    pub name: String,
    pub mode: String,
    pub dmx_from: u16,
    pub dmx_to: u16,
    pub dmx_default: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualTarget {
    pub fixture_id: String,
    pub attribute: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualEdit {
    pub fixture_id: String,
    pub attribute: String,
    pub value: ManualValue,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ManualValue {
    Release {},
    Normalized { value: u16 },
    Function { function_key: String, position: u16 },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualLimits {
    pub manual_changes: usize,
    pub request_bytes: usize,
}
