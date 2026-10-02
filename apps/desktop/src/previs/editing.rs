//! Editing projection stays separate from the immutable background observation.
use super::{Cached, Context, Failure, MAX_SCENE_BYTES, SceneResponse};
use crate::previs::protocol::{Frame, Source, Stamp};
use axum::{
    Json,
    http::header,
    response::{IntoResponse, Response},
};
use std::{sync::Arc, time::Instant};

pub(super) fn scene_response(context: &Context) -> Result<Response, Failure> {
    let cached = context.cache.lock().map_err(|_| Failure::busy())?.clone();
    let (revision, document) = {
        let session = context.shared.try_lock().map_err(|_| Failure::busy())?;
        let revision = session.previs_revision();
        let document = if cached
            .as_ref()
            .is_some_and(|c| c.version == revision.content)
        {
            None
        } else {
            Some(session.previs_document().map_err(Failure::conflict)?)
        };
        (revision, document)
    };
    let cached = if let Some(document) = document {
        let projected = stagemaster_previs::scene(&document).map_err(Failure::invalid)?;
        Arc::new(Cached {
            version: revision.content,
            scene: projected,
            rig: stagemaster_previs::LightRig::new(&document),
        })
    } else {
        cached.ok_or_else(Failure::busy)?
    };
    let bytes = serde_json::to_vec(&SceneResponse {
        stamp: Stamp::new(&context.bridge_id, revision),
        scene: &cached.scene,
    })
    .map_err(|_| Failure::invalid("场地序列化失败"))?;
    if bytes.len() > MAX_SCENE_BYTES {
        return Err(Failure::invalid("场地预演数据超出 32 MiB 限制"));
    }
    // A concurrent edit must not be reported as a current scene.
    let current = context
        .shared
        .try_lock()
        .map_err(|_| Failure::busy())?
        .previs_revision();
    if current != revision {
        return Err(Failure::conflict("场地已变化，请重新读取"));
    }
    let mut cache = context.cache.lock().map_err(|_| Failure::busy())?;
    if cache.as_ref().is_none_or(|c| c.version <= cached.version) {
        *cache = Some(cached);
    }
    Ok(([(header::CONTENT_TYPE, "application/json")], bytes).into_response())
}
pub(super) fn frame_response(context: &Context) -> Result<Json<Frame>, Failure> {
    let cached = context
        .cache
        .lock()
        .map_err(|_| Failure::busy())?
        .clone()
        .ok_or_else(|| Failure::conflict("请先读取场地"))?;
    let input = context
        .shared
        .try_lock()
        .map_err(|_| Failure::busy())?
        .previs_frame()
        .map_err(Failure::invalid)?;
    if cached.version != input.revision.content {
        return Err(Failure::conflict("场地已变化，请重新读取"));
    }
    let (status, mut lights) = match &input.source {
        Source::Defaults => (
            "editing",
            cached.rig.editing(None).map_err(Failure::invalid)?,
        ),
        Source::Scene { scene_id } => match cached.rig.editing(Some(scene_id)) {
            Ok(lights) => ("editing", lights),
            Err(_) => ("missingScene", vec![]),
        },
        Source::Background { .. } => {
            return Err(Failure::conflict("后台来源已切换，请重新读取场地"));
        }
        Source::Playback => {
            let output = input.playback.ok_or_else(Failure::busy)?;
            let lights = output
                .output
                .as_ref()
                .map_or_else(Vec::new, |values| cached.rig.playback(values));
            (output.status, lights)
        }
    };
    if !matches!(input.source, Source::Playback) {
        let factor = f64::from(input.master.effective_percent()) / 100.0;
        for light in &mut lights {
            light.intensity *= factor;
        }
    }
    let mut activity = context.activity.lock().map_err(|_| Failure::busy())?;
    activity.seen = Some(Instant::now());
    activity.problem = match status {
        "missingScene" => Some("原预演场景已删除，请重新选择".into()),
        "stalePlayback" => Some("工程已修改，请重新载入场景列表预览".into()),
        "unloaded" => Some("请先载入一个场景列表".into()),
        _ => None,
    };
    Ok(Json(Frame {
        stamp: Stamp::new(&context.bridge_id, input.revision),
        source: input.source,
        status,
        can_edit: input.can_edit
            && !matches!(status, "stalePlayback" | "missingScene" | "unloaded"),
        lights,
    }))
}
