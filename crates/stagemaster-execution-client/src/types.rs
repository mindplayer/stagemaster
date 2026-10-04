use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Selection {
    Scene { id: String },
    Sequence { id: String },
    Manual {},
    AudioTimeline {},
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub number: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub priority: i16,
    pub selection: Selection,
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub protocol: u8,
    pub execution: String,
    pub mode: String,
    pub physical_output: bool,
    pub project_id: String,
    pub layout: String,
    pub sources: Vec<Source>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<crate::AudioCatalog>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub session_id: String,
    pub expires_ms: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceState {
    pub id: String,
    pub level: u16,
    pub status: Option<String>,
    pub step: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<crate::Progress>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub boot: String,
    pub layout: String,
    pub revision: String,
    pub observed_ms: String,
    pub sources: Vec<SourceState>,
    pub fault: bool,
    pub owner: Option<Owner>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media: Vec<crate::MediaState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<crate::AudioState>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: State,
    pub cycles: String,
    pub missed_periods: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub phase: String,
    pub fault: Option<String>,
    pub snapshot: Option<Snapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outcome {
    pub kind: String,
    pub code: Option<String>,
    pub message: Option<String>,
    pub state: Option<State>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub serial: String,
    pub status: String,
    pub outcome: Option<Outcome>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    Start { step: String },
    Pause {},
    Resume {},
    Next {},
    Stop {},
    Level { value: u16 },
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub host_id: String,
    pub catalog: Catalog,
    pub observation: Observation,
    pub session_id: Option<String>,
    pub controlling: bool,
    pub pending: bool,
    pub record: Option<Record>,
}
