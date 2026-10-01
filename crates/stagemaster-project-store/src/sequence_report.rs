//! Typed handoff exports reuse conflict-aware atomic file commits, without project revisions.
use crate::report_file::ReportFile;
use stagemaster_project::SequenceReport;
use std::path::Path;

pub struct SequenceReportFile(ReportFile);
impl SequenceReportFile {
    /// Capture a native-dialog destination; unrelated CSV files cannot be replaced.
    /// # Errors
    /// Refuses source documents, other formats, symbolic links and non-report existing files.
    pub fn select(path: &Path, source: Option<&Path>) -> Result<Self, String> {
        Ok(Self(ReportFile::select(
            path,
            source,
            "节目单",
            SequenceReport::recognizes,
        )?))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    /// Atomically persist a report produced by the project projection.
    /// # Errors
    /// Rejects changed destination baselines, competing writers and pre-commit I/O failures.
    pub fn save(&mut self, report: &SequenceReport) -> Result<Option<String>, String> {
        self.0.save(report.bytes())
    }
}
