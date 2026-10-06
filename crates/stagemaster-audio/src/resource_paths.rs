//! Managed copies are ordinary paths; an explicitly selected import source is separate.
use std::{fs, path::Path};

pub(crate) fn directory(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err("音乐资源目录不是普通目录，不支持符号链接".into());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("无法检查音乐资源目录：{error}")),
        }
    }
    Ok(())
}
pub(crate) fn file(path: &Path) -> Result<Option<fs::Metadata>, String> {
    directory(path.parent().ok_or("音乐资源位置无效")?)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            Ok(Some(metadata))
        }
        Ok(_) => Err("音乐资源路径不是普通文件，不支持符号链接或文件夹".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("无法检查音乐资源文件：{error}")),
    }
}
