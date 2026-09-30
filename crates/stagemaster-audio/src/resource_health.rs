//! Read-only integrity inspection, independent of decoding and audio transport.
use crate::resources::{adjacent, key};
use crate::{MAX_FILE_BYTES, verify};
use serde::Serialize;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ResourceFileHealth {
    Valid,
    Missing,
    NotSaved,
    Invalid { message: String },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResourceSource {
    Cache,
    Companion,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceHealth {
    pub local: ResourceFileHealth,
    pub companion: ResourceFileHealth,
    pub local_source: Option<ResourceSource>,
}
pub(crate) fn inspect(
    root: &Path,
    digest: &str,
    extension: &str,
    project: Option<&Path>,
    cancelled: &AtomicBool,
) -> Result<ResourceHealth, String> {
    let key = key(digest, extension)?;
    let cache = root.join(&key);
    let companion_path = project.map(|p| adjacent(p).join(&key));
    // Match Resources.resolve: an existing cache file is the playback candidate,
    // even if its digest is wrong. Do not silently choose a different copy here.
    let (path, local_source) = if cache.is_file() {
        (Some(cache.as_path()), Some(ResourceSource::Cache))
    } else if let Some(path) = companion_path.as_ref().filter(|p| p.is_file()) {
        (Some(path.as_path()), Some(ResourceSource::Companion))
    } else {
        (None, None)
    };
    let local = path.map_or(ResourceFileHealth::Missing, |p| {
        inspect_file(p, digest, cancelled)
    });
    let companion = companion_path
        .as_ref()
        .map_or(ResourceFileHealth::NotSaved, |p| {
            if path == Some(p.as_path()) {
                local.clone()
            } else {
                inspect_file(p, digest, cancelled)
            }
        });
    if cancelled.load(Ordering::Relaxed) {
        return Err("音乐资源检查已取消".into());
    }
    Ok(ResourceHealth {
        local,
        companion,
        local_source,
    })
}
fn inspect_file(path: &Path, digest: &str, cancelled: &AtomicBool) -> ResourceFileHealth {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => ResourceFileHealth::Missing,
        Err(e) => ResourceFileHealth::Invalid {
            message: format!("无法读取音乐文件：{e}"),
        },
        Ok(metadata) if !metadata.is_file() => ResourceFileHealth::Invalid {
            message: "音乐路径不是普通文件".into(),
        },
        Ok(metadata) if metadata.len() > MAX_FILE_BYTES => ResourceFileHealth::Invalid {
            message: "音乐文件超过 512 MiB".into(),
        },
        Ok(_) => match verify(path, digest, cancelled) {
            Ok(()) => ResourceFileHealth::Valid,
            Err(message) => ResourceFileHealth::Invalid { message },
        },
    }
}
