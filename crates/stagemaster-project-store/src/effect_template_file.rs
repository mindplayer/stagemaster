//! Portable effect templates use the same atomic destination protection as projects.
use crate::DiskFile;
use stagemaster_project::{EffectTemplateFile, MAX_EFFECT_TEMPLATE_BYTES};
use std::{fs, io::Read, path::Path};

pub struct EffectTemplateFileStore(DiskFile);
impl EffectTemplateFileStore {
    /// Read a regular local file with a bounded buffer; never modifies it.
    /// # Errors
    /// Rejects missing, linked, oversized, malformed or unsupported template files.
    pub fn read(path: &Path) -> Result<EffectTemplateFile, String> {
        let metadata = fs::symlink_metadata(path).map_err(|_| "无法读取灯效模板文件")?;
        if !metadata.is_file() {
            return Err("请选择普通灯效模板文件，不支持链接或文件夹".into());
        }
        if metadata.len() > MAX_EFFECT_TEMPLATE_BYTES as u64 {
            return Err("灯效模板文件超过 16 KiB 限制".into());
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| "无法打开灯效模板文件")?
            .take((MAX_EFFECT_TEMPLATE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "灯效模板文件读取失败")?;
        EffectTemplateFile::decode(&bytes)
    }
    /// Capture an export destination after the native save dialog.
    /// # Errors
    /// Refuses other file extensions, symbolic links and unrelated existing files.
    pub fn select(path: &Path) -> Result<Self, String> {
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.to_ascii_lowercase().ends_with(".smeffect.json"))
        {
            return Err("灯效模板文件名须以 .smeffect.json 结尾".into());
        }
        let target = DiskFile::select(path)?;
        if let Some(bytes) = &target.baseline {
            EffectTemplateFile::decode(bytes)
                .map_err(|_| "所选位置已有其他或不兼容文件，请另取灯效模板文件名")?;
        }
        Ok(Self(target))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    /// Persist an inspected template without changing any project save revision.
    /// # Errors
    /// Rejects changed baselines, competing writers and pre-commit I/O failures.
    pub fn save(&mut self, template: &EffectTemplateFile) -> Result<Option<String>, String> {
        self.0.write_bytes(template.encode()?)
    }
}
