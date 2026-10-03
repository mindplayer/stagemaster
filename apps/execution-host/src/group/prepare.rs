use super::{catalog, manifest::Manifest};
#[cfg(feature = "audio")]
use crate::directory::Directory;
use crate::preparation::Prepared;
use stagemaster_live::Session;
use stagemaster_live_host::{Live, LiveBackend};
use stagemaster_project_store::DiskFile;
use stagemaster_runtime_host::{Configuration, Host};
use std::path::Path;
use uuid::Uuid;

pub(crate) fn prepare(
    project: &Path,
    manifest: &Path,
    #[cfg(feature = "audio")] directory: &Directory,
    #[cfg(feature = "audio")] audio_scope: Option<&stagemaster_audio::OutputScope>,
) -> Result<Prepared<Live>, String> {
    let manifest = Manifest::load(manifest)?;
    let specs = manifest.specs()?;
    let (document, _) = DiskFile::open(project)?;
    #[cfg(feature = "audio")]
    if let Some(config) = manifest.audio {
        return super::audio::prepare(
            document,
            &specs,
            config.output,
            project,
            directory,
            audio_scope,
        );
    }
    let boot = Uuid::new_v4();
    let session = Session::prepare(&document, *boot.as_bytes(), &specs, 0)?;
    let (adapter, source) = catalog::build(&document, &specs, &session)?;
    let backend = LiveBackend::new(session).map_err(|e| e.to_string())?;
    let host = Host::start_backend(backend, Configuration::default()).map_err(|e| e.to_string())?;
    Ok(Prepared {
        host,
        adapter,
        source,
        boot,
    })
}
