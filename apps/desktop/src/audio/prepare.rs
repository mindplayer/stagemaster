use super::Service;
use serde::{Deserialize, Serialize};
use stagemaster_audio::{Waveform, analyze, verify};
use stagemaster_project::AudioAsset;
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Preparation {
    Import,
    Load,
    Locate,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Prepared {
    asset: AudioAsset,
    waveform: Waveform,
}
#[tauri::command]
pub(crate) fn audio_cancel(app: tauri::AppHandle) {
    app.state::<Service>().cancel();
    // Release the command-owned application handle after the immediate cancellation signal.
    drop(app);
}

#[tauri::command]
pub(crate) async fn audio_prepare(
    app: tauri::AppHandle,
    generation: u32,
    kind: Preparation,
) -> Result<Option<Prepared>, String> {
    let cancellation = app.state::<Service>().cancellation_version();
    tauri::async_runtime::spawn_blocking(move || prepare(&app, generation, kind, cancellation))
        .await
        .map_err(|_| "音乐准备未完成".to_string())?
}
fn prepare(
    app: &tauri::AppHandle,
    generation: u32,
    kind: Preparation,
    cancellation: u64,
) -> Result<Option<Prepared>, String> {
    let service = app.state::<Service>();
    let _guard = service.begin_preparation(cancellation)?;
    let session = app.state::<crate::previs::SharedSession>();
    let (doc, project, intent) = {
        let s = session.lock().map_err(|_| "工程会话发生错误")?;
        (
            s.check_snapshot(generation)?,
            s.export_source(generation)?,
            s.audio_load_intent(generation)?,
        )
    };
    let previous = doc.audio_timeline();
    if kind == Preparation::Import && previous.is_some() {
        return Err("请先移除当前音乐".into());
    }
    if kind != Preparation::Import && previous.is_none() {
        return Err("工程还没有音乐".into());
    }
    let (path, digest, extension, name) = if kind == Preparation::Load {
        let asset = &previous.as_ref().ok_or("工程还没有音乐")?.asset;
        let path =
            service
                .resources
                .resolve(&asset.digest, &asset.extension, project.as_deref())?;
        verify(&path, &asset.digest, &service.cancelled)?;
        (
            path,
            asset.digest.clone(),
            asset.extension.clone(),
            asset.file_name.clone(),
        )
    } else {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title(if kind == Preparation::Import {
                "导入音乐"
            } else {
                "重新定位原音乐"
            })
            .add_filter("音频文件", &["wav", "mp3", "flac"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let source = selected.into_path().map_err(|_| "请选择本机音乐文件")?;
        let extension = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let expected = previous.as_ref().map(|t| t.asset.digest.as_str());
        let (digest, path) =
            service
                .resources
                .import(&source, &extension, expected, &service.cancelled)?;
        let name = source
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("音乐文件名无效")?
            .to_owned();
        (path, digest, extension, name)
    };
    let waveform = load_waveform(&service, &path, &digest)?;
    if service.cancelled.load(Ordering::Acquire) {
        return Err("音乐准备已取消".into());
    }
    let asset = AudioAsset {
        digest: digest.clone(),
        extension,
        file_name: name,
        duration_ms: waveform.duration_ms,
    };
    if let Some(track) = &previous
        && waveform.duration_ms != track.asset.duration_ms
    {
        return Err("音乐时长与工程记录不一致，请检查原文件".into());
    }
    let prepared = intent
        .map(|i| i.prepare(path, &service.cancelled))
        .transpose()?;
    let mut s = session.lock().map_err(|_| "工程会话发生错误")?;
    service.check_cancelled()?;
    service.check_cancellation_version(cancellation)?;
    s.check_snapshot(generation)?;
    if let Some(prepared) = prepared {
        s.apply_audio_load(generation, prepared)?;
    }
    *service.waveform.lock().map_err(|_| "波形缓存不可用")? = Some((digest, waveform.clone()));
    Ok(Some(Prepared { asset, waveform }))
}

fn load_waveform(
    service: &Service,
    path: &std::path::Path,
    digest: &str,
) -> Result<Waveform, String> {
    let cached = service
        .waveform
        .lock()
        .map_err(|_| "波形缓存不可用")?
        .as_ref()
        .filter(|(key, _)| key == digest)
        .map(|(_, w)| w.clone());
    cached.map_or_else(|| analyze(path, &service.cancelled), Ok)
}
