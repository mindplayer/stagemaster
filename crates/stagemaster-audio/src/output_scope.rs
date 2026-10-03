//! Cooperative process ownership, separate from operator authority and device routing.
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    path::PathBuf,
};

/// One application-configured output route. Never derive this directory from show content.
#[derive(Clone)]
pub struct OutputScope {
    directory: PathBuf,
}

/// A unique reservation. Closing the handle (including process death) releases ownership.
/// The lock file is deliberately never removed and the handle is never cloned or inherited.
pub struct OutputLease {
    _file: File,
}

impl OutputScope {
    /// # Errors
    /// Reject relative scope paths. Configuration alone does not reserve or open audio.
    pub fn new(directory: PathBuf) -> Result<Self, String> {
        if !directory.is_absolute() {
            return Err("声音输出占用目录必须是绝对路径".into());
        }
        Ok(Self { directory })
    }

    /// # Errors
    /// Reject a busy route or an unavailable/non-private local coordination directory.
    pub fn reserve(&self) -> Result<OutputLease, String> {
        self.prepare_directory()?;
        let path = self.directory.join("audio-output.lock");
        match fs::symlink_metadata(&path) {
            Ok(metadata) => validate_file(&metadata)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("无法检查声音输出占用：{error}")),
        }
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path).map_err(|e| e.to_string())?;
        validate_file(&file.metadata().map_err(|e| e.to_string())?)?;
        validate_file(&fs::symlink_metadata(path).map_err(|e| e.to_string())?)?;
        file.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => {
                "声音输出已被其他窗口或后台占用，请先在原窗口停止音乐或关闭后台".into()
            }
            TryLockError::Error(error) => format!("无法取得声音输出占用：{error}"),
        })?;
        Ok(OutputLease { _file: file })
    }

    #[cfg(unix)]
    fn prepare_directory(&self) -> Result<(), String> {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        match fs::DirBuilder::new().mode(0o700).create(&self.directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("无法准备声音输出占用目录：{error}")),
        }
        let meta = fs::symlink_metadata(&self.directory).map_err(|e| e.to_string())?;
        if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
            return Err("声音输出占用目录须为私有目录且不能是符号链接".into());
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn prepare_directory(&self) -> Result<(), String> {
        Err("当前系统的声音输出占用目录尚未验证".into())
    }
}

fn validate_file(meta: &fs::Metadata) -> Result<(), String> {
    if !meta.is_file() {
        return Err("声音输出占用记录不是普通文件".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err("声音输出占用记录须为私有文件".into());
        }
    }
    Ok(())
}
