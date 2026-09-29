use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
pub const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
const CACHE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
#[derive(Clone)]
pub struct Resources {
    root: PathBuf,
}
impl Resources {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    /// # Errors
    /// Reject unsafe resource keys or absent local resources.
    pub fn resolve(
        &self,
        digest: &str,
        extension: &str,
        project: Option<&Path>,
    ) -> Result<PathBuf, String> {
        let key = key(digest, extension)?;
        let candidates = [
            Some(self.root.join(&key)),
            project.map(|p| adjacent(p).join(&key)),
        ];
        candidates
            .into_iter()
            .flatten()
            .find(|p| p.is_file())
            .ok_or_else(|| {
                "找不到音乐文件，请点“重新定位音乐”选择原文件；移动工程时须带上同名 .assets 文件夹"
                    .into()
            })
    }
    /// Copy and hash through an internal fixed buffer. Publication is atomic, no source is overwritten.
    /// # Errors
    /// Reject excess files, cancellation, invalid keys, I/O failure or content mismatch.
    pub fn import(
        &self,
        source: &Path,
        extension: &str,
        expected: Option<&str>,
        cancelled: &AtomicBool,
    ) -> Result<(String, PathBuf), String> {
        key(&"0".repeat(64), extension)?;
        fs::create_dir_all(&self.root).map_err(|e| io_error(&e))?;
        let size = fs::metadata(source).map_err(|e| io_error(&e))?.len();
        if size == 0 || size > MAX_FILE_BYTES {
            return Err("音乐文件需要在 1 字节至 512 MiB 之间".into());
        }
        let occupied = fs::read_dir(&self.root)
            .map_err(|e| io_error(&e))?
            .filter_map(Result::ok)
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .fold(0_u64, u64::saturating_add);
        if occupied.saturating_add(size) > CACHE_BYTES {
            return Err("本机音乐缓存已达到 4 GiB，请整理音乐资源后再导入".into());
        }
        let mut temporary =
            tempfile::NamedTempFile::new_in(&self.root).map_err(|e| io_error(&e))?;
        let digest = copy_hash(source, Some(temporary.as_file_mut()), cancelled)?;
        if expected.is_some_and(|e| e != digest) {
            return Err("选择的音乐内容与原工程不同，请选择原文件".into());
        }
        temporary.as_file().sync_all().map_err(|e| io_error(&e))?;
        let target = self.root.join(key(&digest, extension)?);
        if target.exists() {
            if verify(&target, &digest, cancelled).is_err() {
                if cancelled.load(Ordering::Relaxed) {
                    return Err("音乐准备已取消".into());
                }
                // Repair only our content-addressed cache from the already verified temporary.
                temporary.persist(&target).map_err(|e| io_error(&e.error))?;
            }
        } else {
            temporary
                .persist_noclobber(&target)
                .map_err(|e| io_error(&e.error))?;
        }
        Ok((digest, target))
    }
    /// Archive assets before the project JSON is committed. Valid existing content is reused; corrupt managed copies can be repaired.
    /// # Errors
    /// Missing/mutated resources and storage failures leave the project uncommitted.
    pub fn archive(
        &self,
        digest: &str,
        extension: &str,
        current: Option<&Path>,
        target: &Path,
    ) -> Result<(), String> {
        let source = self.resolve(digest, extension, current)?;
        let destination = adjacent(target);
        let archived = destination.join(key(digest, extension)?);
        let cancel = AtomicBool::new(false);
        if archived.exists() && verify(&archived, digest, &cancel).is_ok() {
            return Ok(());
        }
        Self::new(destination).import(&source, extension, Some(digest), &cancel)?;
        Ok(())
    }
}
/// # Errors
/// Reject content mismatch, oversized resources, cancellation and read failure.
pub fn verify(path: &Path, expected: &str, cancelled: &AtomicBool) -> Result<(), String> {
    if copy_hash(path, None, cancelled)? != expected {
        return Err("音乐文件内容已经改变，请重新定位原文件".into());
    }
    Ok(())
}
fn key(digest: &str, extension: &str) -> Result<String, String> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || !["wav", "mp3", "flac"].contains(&extension)
    {
        return Err("音乐资源标识无效".into());
    }
    Ok(format!("{digest}.{extension}"))
}
fn adjacent(project: &Path) -> PathBuf {
    let mut path = project.as_os_str().to_os_string();
    path.push(".assets");
    PathBuf::from(path)
}
fn copy_hash(
    source: &Path,
    mut target: Option<&mut File>,
    cancelled: &AtomicBool,
) -> Result<String, String> {
    let mut reader = File::open(source).map_err(|e| io_error(&e))?;
    if !reader.metadata().map_err(|e| io_error(&e))?.is_file() {
        return Err("请选择普通音乐文件".into());
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    let mut total = 0_u64;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("音乐准备已取消".into());
        }
        let count = reader.read(&mut buffer).map_err(|e| io_error(&e))?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > MAX_FILE_BYTES {
            return Err("音乐文件超过 512 MiB".into());
        }
        digest.update(&buffer[..count]);
        if let Some(file) = target.as_deref_mut() {
            file.write_all(&buffer[..count]).map_err(|e| io_error(&e))?;
        }
    }
    if total == 0 {
        return Err("音乐文件为空".into());
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn io_error(error: &std::io::Error) -> String {
    format!("音乐资源读写失败：{error}")
}
