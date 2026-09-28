use serde::Serialize;
use stagemaster_package::{Archive, Error, ReadAt};
use stagemaster_transfer::Upload;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Bytes(Arc<[u8]>);
impl ReadAt for Bytes {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        self.0.as_ref().read_exact(offset, target)
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo {
    pub project_name: String,
    pub project_id: String,
    pub revision_id: String,
    pub digest: String,
    pub bytes: usize,
    pub programs: usize,
    pub loader_bytes: usize,
}
/// Fully checked, immutable source; prepare on the application's blocking work queue.
pub struct Prepared {
    pub(crate) info: PackageInfo,
    pub(crate) upload: Upload<Bytes>,
}
impl Prepared {
    /// # Errors
    /// Reject corrupt, unsupported or excessive packages before any device mutation.
    pub fn new(bytes: Arc<[u8]>) -> Result<Self, String> {
        let archive = Archive::open(bytes.as_ref()).map_err(|e| e.to_string())?;
        let source = archive.source();
        let info = PackageInfo {
            project_name: source.project_name.clone(),
            project_id: hex(&source.project_id),
            revision_id: hex(&source.revision_id),
            digest: hex(archive.digest()),
            bytes: archive.total_bytes(),
            programs: archive.entries().len(),
            loader_bytes: archive
                .entries()
                .iter()
                .map(|p| p.usage.loader_peak_bytes)
                .max()
                .unwrap_or(0),
        };
        let upload = Upload::new(Bytes(bytes)).map_err(|e| e.to_string())?;
        Ok(Self { info, upload })
    }
    #[must_use]
    pub const fn info(&self) -> &PackageInfo {
        &self.info
    }
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
