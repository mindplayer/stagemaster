//! Atomic package export reuses the project's conflict-aware file commit primitive.
use crate::DiskFile;
use stagemaster_package::Archive;
use std::path::Path;
pub struct PackageFile(DiskFile);
impl PackageFile {
    /// Capture a user-selected destination after the native dialog. Existing files must be packages.
    /// # Errors
    /// Refuses project paths, symbolic links, other extensions and unrelated existing files.
    pub fn select(path: &Path, source: Option<&Path>) -> Result<Self, String> {
        if !path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("smpkg"))
        {
            return Err("播放包文件名须以 .smpkg 结尾".into());
        }
        let target = DiskFile::select(path)?;
        if let Some(source) = source {
            let normalized = source
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .zip(source.file_name())
                .map(|(p, n)| p.join(n));
            if normalized.as_deref() == Some(target.path()) {
                return Err("播放包不能覆盖当前工程文件，请另选位置".into());
            }
        }
        if let Some(bytes) = &target.baseline {
            Archive::open(bytes.as_slice())
                .map_err(|_| "所选位置已有其他文件或损坏包，请使用新文件名")?;
        }
        Ok(Self(target))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    /// Atomically write a complete independently verified archive without creating a project revision.
    /// # Errors
    /// Rejects invalid packages, changed destination baselines, competing writers and I/O failures.
    pub fn save(&mut self, bytes: &[u8]) -> Result<Option<String>, String> {
        Archive::open(bytes).map_err(|e| e.to_string())?;
        self.0.write_bytes(bytes.to_vec())
    }
}
