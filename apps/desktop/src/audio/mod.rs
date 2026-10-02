mod loading;
mod looping;
mod performing;
mod preparation_guard;
pub(crate) use loading::{LoadIntent, PreparedLoad};
pub(crate) mod prepare;
mod preview;
pub(crate) use preview::AudioPreview;
use serde::Deserialize;
use stagemaster_audio::{Position, Resources, Waveform};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64},
};
pub(crate) struct Service {
    pub resources: Resources,
    preparing: AtomicBool,
    cancelled: AtomicBool,
    cancellation: AtomicU64,
    waveform: Mutex<Option<(String, Waveform)>>,
}
impl Service {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            resources: Resources::new(root),
            preparing: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            cancellation: AtomicU64::new(0),
            waveform: Mutex::new(None),
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Command {
    Snapshot,
    Play,
    Pause,
    Stop,
    Volume {
        percent: u8,
    },
    Seek {
        position_ms: u64,
    },
    SetLoop {
        range: Option<stagemaster_audio::LoopRange>,
    },
    ExitLoop {
        instance: String,
        region_id: String,
        pass: String,
        requested: bool,
    },
}
#[tauri::command]
pub(crate) async fn audio_request(
    app: tauri::AppHandle,
    generation: u32,
    command: Command,
) -> Result<Position, String> {
    use tauri::Manager;
    let service = app.state::<Service>();
    let cancellation = service.cancellation_version();
    tauri::async_runtime::spawn_blocking(move || {
        if let Command::SetLoop { range } = &command {
            return looping::configure(&app, generation, *range, cancellation);
        }
        performing::execute(&app, generation, command, cancellation)
    })
    .await
    .map_err(|_| "音频操作未完成".to_string())?
}

#[cfg(test)]
mod tests;
