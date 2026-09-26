use serde::{Deserialize, Serialize};
use stagemaster_project::FixturePlacement;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Source {
    #[default]
    Defaults,
    Scene {
        scene_id: String,
    },
    Playback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Revision {
    pub generation: u32,
    pub content: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Stamp {
    pub protocol: u8,
    pub bridge_id: String,
    pub generation: u32,
    pub version: String,
}
impl Stamp {
    pub(super) fn new(id: &str, revision: Revision) -> Self {
        Self {
            protocol: 1,
            bridge_id: id.into(),
            generation: revision.generation,
            version: revision.content.to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PlacementRequest {
    pub bridge_id: String,
    pub generation: u32,
    pub version: String,
    pub placement: FixturePlacement,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Status,
    Enable,
    Disable,
    Source { generation: u32, source: Source },
    Editing { generation: u32, allowed: bool },
}

pub(crate) struct InputFrame {
    pub revision: Revision,
    pub source: Source,
    pub can_edit: bool,
    pub playback: Option<crate::preview::RenderOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Frame {
    #[serde(flatten)]
    pub stamp: Stamp,
    pub source: Source,
    pub status: &'static str,
    pub can_edit: bool,
    pub lights: Vec<stagemaster_previs::Light>,
}
