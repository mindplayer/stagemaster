mod looping;
pub(crate) mod prepare;
mod preview;
pub(crate) use preview::AudioPreview;
use serde::Deserialize;
use stagemaster_audio::{Position, Resources, Waveform};
use std::sync::{Mutex, atomic::AtomicBool};
use tauri::Manager;
pub(crate) struct Service {
    pub resources: Resources,
    preparing: AtomicBool,
    loop_preparing: AtomicBool,
    cancelled: AtomicBool,
    waveform: Mutex<Option<(String, Waveform)>>,
}
impl Service {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            resources: Resources::new(root),
            preparing: AtomicBool::new(false),
            loop_preparing: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            waveform: Mutex::new(None),
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
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
}
#[tauri::command]
pub(crate) async fn audio_request(
    app: tauri::AppHandle,
    generation: u32,
    command: Command,
) -> Result<Position, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Command::SetLoop { range } = command {
            return looping::configure(&app, generation, range);
        }

        app.state::<crate::previs::SharedSession>()
            .lock()
            .map_err(|_| "工程会话发生错误")?
            .audio_request(generation, command)
    })
    .await
    .map_err(|_| "音频操作未完成".to_string())?
}

#[cfg(test)]
mod tests;
