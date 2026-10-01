//! Common conflict-aware destination policy; typed wrappers own format recognition.
use crate::DiskFile;
use std::path::Path;

pub(crate) struct ReportFile(DiskFile);
impl ReportFile {
    pub(crate) fn select(
        path: &Path,
        source: Option<&Path>,
        label: &str,
        recognizes: fn(&[u8]) -> bool,
    ) -> Result<Self, String> {
        if !path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("csv"))
        {
            return Err(format!("{label}文件名须以 .csv 结尾"));
        }
        let target = DiskFile::select(path)?;
        let normalized_source = source.and_then(|p| {
            p.parent()
                .and_then(|p| p.canonicalize().ok())
                .zip(p.file_name())
                .map(|(parent, name)| parent.join(name))
        });
        if normalized_source.as_deref() == Some(target.path()) {
            return Err(format!("{label}不能覆盖当前工程，请另选位置"));
        }
        if target.baseline.as_deref().is_some_and(|b| !recognizes(b)) {
            return Err(format!("所选位置已有其他文件，请为{label}使用新文件名"));
        }
        Ok(Self(target))
    }
    pub(crate) fn path(&self) -> &Path {
        self.0.path()
    }
    pub(crate) fn save(&mut self, bytes: &[u8]) -> Result<Option<String>, String> {
        self.0.write_bytes(bytes.to_vec())
    }
}
