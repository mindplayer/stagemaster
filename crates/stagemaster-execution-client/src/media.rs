//! Typed observations for the same controlled media group; these are not sound-card feedback.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioOutput {
    Software,
    SystemDefault,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioCatalog {
    pub output: AudioOutput,
    pub duration_ms: u64,
    pub group: String,
    pub seek_includes_end: bool,
    #[serde(default)]
    pub performance_loops: bool,
    #[serde(default)]
    pub provider_recovery: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaCompletion {
    Pending,
    Applied,
    Failed,
    TimedOut,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaControl {
    pub request: String,
    pub status: MediaCompletion,
    pub problem: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaStatus {
    Ready,
    Following,
    Paused,
    Lost,
    Stopped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EndReason {
    Ended,
    Failed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaTermination {
    pub generation: String,
    pub reason: EndReason,
    pub applied: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    pub id: String,
    pub generation: String,
    pub status: MediaStatus,
    pub position_ms: u64,
    pub control: Option<MediaControl>,
    pub termination: Option<MediaTermination>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioStatus {
    Ready,
    Preparing,
    Playing,
    Paused,
    Stopped,
    Ended,
    Failed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioState {
    pub output: AudioOutput,
    pub status: AudioStatus,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub instance: Option<String>,
    pub frames: String,
    pub problem: Option<String>,
    pub loop_state: Option<LoopState>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopState {
    pub region: usize,
    pub name: String,
    pub pass: String,
    pub exit_requested: bool,
    pub pending_exit: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MediaAction {
    Play {},
    Pause {},
    Stop {},
    Recover {
        position_ms: u64,
    },
    Seek {
        position_ms: u64,
        playing: bool,
    },
    ExitLoop {
        instance: String,
        region: usize,
        pass: String,
        requested: bool,
    },
}
