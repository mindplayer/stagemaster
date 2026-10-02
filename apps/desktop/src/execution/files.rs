use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub(super) fn private_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::{
            fs::DirBuilder,
            os::unix::fs::{DirBuilderExt, PermissionsExt},
        };
        if !path.exists() {
            DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(|e| e.to_string())?;
        }
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !meta.is_dir() || meta.file_type().is_symlink() || meta.permissions().mode() & 0o077 != 0
        {
            return Err("后台数据目录须为当前用户的私有目录".into());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    Err("当前系统的后台数据目录尚未验证".into())
}
pub(super) fn create(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn read_run(root: &Path) -> Result<Option<PathBuf>, String> {
    let path = root.join("current");
    if !path.exists() {
        return Ok(None);
    }
    let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 36 {
        return Err("后台运行记录无效".into());
    }
    let mut name = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(37)
        .read_to_string(&mut name)
        .map_err(|e| e.to_string())?;
    let id = Uuid::parse_str(&name).map_err(|_| "后台运行身份无效")?;
    if id.is_nil() || id.to_string() != name {
        return Err("后台运行身份无效".into());
    }
    Ok(Some(root.join(name)))
}
pub(super) fn record(root: &Path, name: &str) -> Result<(), String> {
    create(&root.join("current"), name.as_bytes())?;
    File::open(root)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn lock(root: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(root.join("manager.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|_| "另一个应用正在管理此后台")?;
    Ok(file)
}
pub(super) fn ended(run: &Path) -> bool {
    let Ok(file) = OpenOptions::new()
        .write(true)
        .open(run.join("host/lifetime.lock"))
    else {
        return false;
    };
    file.try_lock().is_ok()
}
pub(super) fn clear(root: &Path) -> Result<(), String> {
    fs::remove_file(root.join("current")).map_err(|e| e.to_string())
}
