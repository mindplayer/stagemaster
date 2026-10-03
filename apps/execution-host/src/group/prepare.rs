use super::{catalog, manifest::Manifest};
use crate::{
    directory::Directory,
    media::{Owner, Setup},
    preparation::Prepared,
};
use serde_json::json;
use stagemaster_live::{PlaybackSelection, Session};
use stagemaster_live_host::{Live, LiveBackend, media::ControlSpec};
use stagemaster_project_store::DiskFile;
use stagemaster_runtime_host::{Configuration, Host};
use std::path::Path;
use uuid::Uuid;

pub(crate) fn prepare(
    project: &Path,
    manifest: &Path,
    directory: &Directory,
) -> Result<Prepared<Live>, String> {
    let manifest = Manifest::load(manifest)?;
    let specs = manifest.specs()?;
    let (document, _) = DiskFile::open(project)?;
    let setup = manifest
        .audio
        .map(|config| {
            let source = specs
                .iter()
                .find(|s| matches!(s.playback, Some(PlaybackSelection::AudioTimeline)))
                .ok_or("音乐来源不存在")?;
            Setup::prepare(&document, project, directory, source.id, config.output)
        })
        .transpose()?;
    let boot = Uuid::new_v4();
    let session = if let Some(setup) = &setup {
        Session::prepare_with_media(
            &document,
            *boot.as_bytes(),
            &specs,
            std::slice::from_ref(&setup.group),
            0,
        )?
    } else {
        Session::prepare(&document, *boot.as_bytes(), &specs, 0)?
    };
    let (mut adapter, mut source) = catalog::build(&document, &specs, &session)?;
    let (backend, provider) = if let Some(setup) = setup {
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
        source["audio"] = json!({"output":setup.output,"durationMs":setup.duration_ms,"group":Uuid::from_bytes(setup.group.id).to_string(),"seekIncludesEnd":false});
        source["capabilities"]
            .as_array_mut()
            .ok_or("能力目录无效")?
            .push(json!("backgroundLinearAudio"));
        (backend, Some((setup, prepare, ports.remove(0), target)))
    } else {
        (LiveBackend::new(session).map_err(|e| e.to_string())?, None)
    };
    let host = Host::start_backend(backend, Configuration::default()).map_err(|e| e.to_string())?;
    if let Some((setup, prepare, port, target)) = provider {
        adapter.media = Some(Owner::start(setup, document, prepare, port, &host, target)?);
    }
    Ok(Prepared {
        host,
        adapter,
        source,
        boot,
    })
}
