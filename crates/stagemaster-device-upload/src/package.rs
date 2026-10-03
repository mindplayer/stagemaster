use serde::Serialize;
use stagemaster_delivery::Package;
use stagemaster_transfer::Upload;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo {
    pub execution_semantics: u16,
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
    pub(crate) upload: Upload<Package>,
}
impl Prepared {
    /// # Errors
    /// Reject corrupt, unsupported or excessive packages before any device mutation.
    pub fn new(bytes: Arc<[u8]>) -> Result<Self, String> {
        Self::from_package(Package::from_bytes(bytes, None).map_err(|e| e.to_string())?)
    }
    /// Reuse a checked local/HTTPS import without granting installation or playback rights.
    /// # Errors
    /// Report upload preparation failure; target capabilities are still checked at start.
    pub fn from_package(package: Package) -> Result<Self, String> {
        let archive = package.archive();
        let source = archive.source();
        let info = PackageInfo {
            execution_semantics: archive.semantics(),
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
        let upload = Upload::new(package).map_err(|e| e.to_string())?;
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
