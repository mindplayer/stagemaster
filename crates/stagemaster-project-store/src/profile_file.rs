//! Portable fixture files use the same atomic destination protection as projects.
use crate::DiskFile;
use stagemaster_project::{MAX_PROFILE_FILE_BYTES, ProfileFile};
use std::{fs, io::Read, path::Path};

pub struct ProfileFileStore(DiskFile);
impl ProfileFileStore {
    /// Read a regular local file with a bounded buffer; never modifies it.
    /// # Errors
    /// Rejects missing, linked, oversized, malformed or unsupported mode files.
    pub fn read(path: &Path) -> Result<ProfileFile, String> {
        let metadata = fs::symlink_metadata(path).map_err(|_| "无法读取灯具模式文件")?;
        if !metadata.is_file() {
            return Err("请选择普通灯具模式文件，不支持链接或文件夹".into());
        }
        if metadata.len() > MAX_PROFILE_FILE_BYTES as u64 {
            return Err("灯具模式文件超过 512 KiB 限制".into());
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| "无法打开灯具模式文件")?
            .take((MAX_PROFILE_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "灯具模式文件读取失败")?;
        ProfileFile::decode(&bytes)
    }
    /// Capture an export destination after the native save dialog.
    /// # Errors
    /// Refuses other file extensions, symbolic links and unrelated existing files.
    pub fn select(path: &Path) -> Result<Self, String> {
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.to_ascii_lowercase().ends_with(".smfixture.json"))
        {
            return Err("灯具模式文件名须以 .smfixture.json 结尾".into());
        }
        let target = DiskFile::select(path)?;
        if let Some(bytes) = &target.baseline {
            ProfileFile::decode(bytes)
                .map_err(|_| "所选位置已有其他或不兼容文件，请另取模式文件名")?;
        }
        Ok(Self(target))
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    /// Persist an inspected mode without changing any project save revision.
    /// # Errors
    /// Rejects changed baselines, competing writers and pre-commit I/O failures.
    pub fn save(&mut self, profile: &ProfileFile) -> Result<Option<String>, String> {
        self.0.write_bytes(profile.encode()?)
    }
}
