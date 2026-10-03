//! Optional native-audio assembly; uses the same host, control authority and playback semantics.
use super::catalog;
use crate::{
    directory::Directory,
    media::{OutputKind, Owner, Setup},
    preparation::Prepared,
};
use serde_json::json;
use stagemaster_live::{PlaybackSelection, Session, SourceSpec};
use stagemaster_live_host::{Live, LiveBackend, media::ControlSpec};
use stagemaster_project::Document;
use stagemaster_runtime_host::{Configuration, Host};
use std::path::Path;
use uuid::Uuid;

pub(super) fn prepare(
    document: Document,
    specs: &[SourceSpec],
    output: OutputKind,
    project: &Path,
    directory: &Directory,
    audio_scope: Option<&stagemaster_audio::OutputScope>,
) -> Result<Prepared<Live>, String> {
    let source = specs
        .iter()
        .find(|s| matches!(s.playback, Some(PlaybackSelection::AudioTimeline)))
        .ok_or("音乐来源不存在")?;
    let setup = Setup::prepare(
        &document,
        project,
        directory,
        source.id,
        output,
        audio_scope,
    )?;
    let boot = Uuid::new_v4();
    let session = Session::prepare_with_media(
        &document,
        *boot.as_bytes(),
        specs,
        std::slice::from_ref(&setup.group),
        0,
    )?;
    let (mut adapter, mut source) = catalog::build(&document, specs, &session)?;
    let key = session.media_key(setup.group.id).ok_or("媒体组不存在")?;
    let prepare = session.media_preparer(key)?;
    let target = session.host_clock();
    let (backend, mut ports) = LiveBackend::with_controlled_media(
        session,
        &[ControlSpec {
            group: setup.group.id,
            duration_ms: setup.duration_ms,
            timeout_ms: 5000,
        }],
    )
    .map_err(|e| e.to_string())?;
    source["audio"] = json!({"output":setup.output,"durationMs":setup.duration_ms,"group":Uuid::from_bytes(setup.group.id).to_string(),"seekIncludesEnd":true,"performanceLoops":setup.loops,"providerRecovery":true});
    if setup.loops {
        source["capabilities"]
            .as_array_mut()
            .ok_or("能力目录无效")?
            .push(json!("backgroundAudioLoops"));
    }
    source["capabilities"]
        .as_array_mut()
        .ok_or("能力目录无效")?
        .extend([
            json!("backgroundLinearAudio"),
            json!("backgroundAudioRecovery"),
        ]);
    let host = Host::start_backend(backend, Configuration::default()).map_err(|e| e.to_string())?;
    adapter.media = Some(Owner::start(
        setup,
        document,
        prepare,
        ports.remove(0),
        &host,
        target,
    )?);
    Ok(Prepared {
        host,
        adapter,
        source,
        boot,
    })
}
