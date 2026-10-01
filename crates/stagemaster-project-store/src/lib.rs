//! Local file persistence, separate from project semantics and desktop dialogs.
mod effect_template_file;
mod package;
mod patch_report;
mod profile_file;
pub use effect_template_file::EffectTemplateFileStore;
mod recovery;
mod report_file;
mod sequence_report;
pub use package::PackageFile;
pub use patch_report::PatchReportFile;
pub use profile_file::ProfileFileStore;
pub use recovery::{
    MAX_RECOVERY_RECORDS, RecoveryCandidate, RecoveryCatalog, RecoveryEntry, RecoverySession,
    RecoveryState, RecoveryStore,
};
pub use sequence_report::SequenceReportFile;
use stagemaster_project::{Document, MAX_BYTES};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct DiskFile {
    path: PathBuf,
    baseline: Option<Vec<u8>>,
}
pub struct SaveReceipt {
    pub document: Document,
    pub warning: Option<String>,
}

impl DiskFile {
    /// Open a complete validated project without changing the active session.
    /// # Errors
    /// Rejects missing, oversized, non-regular or invalid project files.
    pub fn open(path: &Path) -> Result<(Document, Self), String> {
        let target = Self::select(path)?;
        let bytes = target.baseline.as_deref().ok_or("工程文件不存在")?;
        let document = Document::decode(bytes)?;
        Ok((document, target))
    }
    /// Capture the destination immediately after an authorized native save dialog.
    /// # Errors
    /// Rejects unreadable destinations and non-regular existing files.
    pub fn select(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or("保存目录无效")?
            .canonicalize()
            .map_err(|_| "保存目录不存在")?;
        let name = path.file_name().ok_or("文件名无效")?;
        let path = parent.join(name);
        let baseline = read_current(&path)?;
        Ok(Self { path, baseline })
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Commit a complete new revision, then update the disk baseline.
    /// # Errors
    /// Rejects concurrent changes, competing writers and all pre-commit I/O failures.
    pub fn save(&mut self, draft: &Document) -> Result<SaveReceipt, String> {
        let next = draft.next_revision();
        let warning = self.write_bytes(next.encode()?)?;
        Ok(SaveReceipt {
            document: next,
            warning,
        })
    }
    fn write_bytes(&mut self, bytes: Vec<u8>) -> Result<Option<String>, String> {
        let parent = self.path.parent().ok_or("保存目录无效")?;
        let mut lock_name = self.path.file_name().ok_or("文件名无效")?.to_os_string();
        lock_name.push(".stagemaster-lock");
        let lock_path = parent.join(lock_name);
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            return Err("保存锁文件无效".into());
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .map_err(|_| "无法创建保存锁，请检查目录权限")?;
        lock.try_lock()
            .map_err(|_| "此文件正在被另一个舞台大师进程保存，请稍后重试")?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".stagemaster-")
            .suffix(".tmp")
            .tempfile_in(parent)
            .map_err(|_| "无法创建临时文件，请检查目录权限")?;
        if read_current(&self.path)? != self.baseline {
            return Err("磁盘文件已被其他程序修改或删除。请另存为新文件，或重新打开后编辑".into());
        }
        if let Ok(metadata) = fs::metadata(&self.path) {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())
                .map_err(|_| "无法保留文件权限")?;
        }
        temporary
            .write_all(&bytes)
            .map_err(|_| "文件写入失败，请检查剩余磁盘空间")?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|_| "文件尚未完成磁盘同步")?;
        // Repeat the comparison after writing. Cooperative writers hold the same stable lock.
        if read_current(&self.path)? != self.baseline {
            return Err("保存期间磁盘文件发生变化，已取消覆盖，请另存为".into());
        }
        if self.baseline.is_some() {
            temporary
                .persist(&self.path)
                .map_err(|_| "无法替换文件，原文件未被本次保存改写")?;
        } else {
            temporary
                .persist_noclobber(&self.path)
                .map_err(|_| "目标文件已出现或无法写入，请重新选择保存位置")?;
        }
        self.baseline = Some(bytes);
        // The rename is the commit point. A post-commit durability warning must not
        // falsely report that the old file is still present or keep a stale baseline.
        let warning = sync_directory(parent)
            .err()
            .map(|()| "文件已写入，但目录同步失败；请检查存储设备后再次保存".into());
        Ok(warning)
    }
}
fn read_current(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("无法读取工程文件信息".into()),
    };
    if !metadata.is_file() {
        return Err("请选择普通工程文件，不支持符号链接或文件夹".into());
    }
    if metadata.len() > MAX_BYTES as u64 {
        return Err("工程超过 8 MiB 限制".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "无法打开工程文件")?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "工程读取失败")?;
    if bytes.len() > MAX_BYTES {
        return Err("工程超过 8 MiB 限制".into());
    }
    Ok(Some(bytes))
}
#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ()> {
    File::open(path).and_then(|f| f.sync_all()).map_err(|_| ())
}
#[cfg(not(unix))]
fn sync_directory(_: &Path) -> Result<(), ()> {
    Ok(())
}
