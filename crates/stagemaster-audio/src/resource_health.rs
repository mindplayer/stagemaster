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
    // Match resolve: an invalid cache must not silently become another playback copy.
    let cached = inspect_file(&cache, digest, cancelled);
    let companion = companion_path
        .as_ref()
        .map_or(ResourceFileHealth::NotSaved, |p| {
            if p == &cache {
                cached.clone()
            } else {
                inspect_file(p, digest, cancelled)
            }
        });
    let (local, local_source) = if cached != ResourceFileHealth::Missing {
        (cached, Some(ResourceSource::Cache))
    } else if companion_path.is_some() && companion != ResourceFileHealth::Missing {
        (companion.clone(), Some(ResourceSource::Companion))
    } else {
        (ResourceFileHealth::Missing, None)
    };
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
    match crate::resource_paths::file(path) {
        Ok(None) => ResourceFileHealth::Missing,
        Err(message) => ResourceFileHealth::Invalid { message },
        Ok(Some(metadata)) if metadata.len() > MAX_FILE_BYTES => ResourceFileHealth::Invalid {
            message: "音乐文件超过 512 MiB".into(),
        },
        Ok(_) => match verify(path, digest, cancelled) {
            Ok(()) => ResourceFileHealth::Valid,
            Err(message) => ResourceFileHealth::Invalid { message },
        },
    }
}
