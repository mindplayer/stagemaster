//! Typed handoff exports reuse conflict-aware atomic file commits, without project revisions.
use crate::DiskFile;
use stagemaster_project::PatchReport;
use std::path::Path;

pub struct PatchReportFile(DiskFile);
impl PatchReportFile {
    /// Capture a native-dialog destination; unrelated CSV files cannot be replaced.
    /// # Errors
    /// Refuses source documents, other formats, symbolic links and non-report existing files.
    pub fn select(path: &Path, source: Option<&Path>) -> Result<Self, String> {
        if !path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("csv"))
        {
            return Err("配灯表文件名须以 .csv 结尾".into());
        }
        let target = DiskFile::select(path)?;
        let normalized_source = source.and_then(|p| {
            p.parent()
                .and_then(|p| p.canonicalize().ok())
                .zip(p.file_name())
                .map(|(parent, name)| parent.join(name))
        });
        if normalized_source.as_deref() == Some(target.path()) {
            return Err("配灯表不能覆盖当前工程，请另选位置".into());
        }
        if target
            .baseline
            .as_deref()
            .is_some_and(|b| !PatchReport::recognizes(b))
        {
            return Err("所选位置已有其他文件，请为配灯表使用新文件名".into());
        }
        Ok(Self(target))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    /// Atomically persist a report produced by the project projection.
    /// # Errors
    /// Rejects changed destination baselines, competing writers and pre-commit I/O failures.
    pub fn save(&mut self, report: &PatchReport) -> Result<Option<String>, String> {
        self.0.write_bytes(report.bytes().to_vec())
    }
}
