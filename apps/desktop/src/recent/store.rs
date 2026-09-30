use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_ENTRIES: usize = 12;
const MAX_BYTES: u64 = 65_536;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Entry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub opened_at_ms: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    version: u8,
    entries: Vec<Entry>,
}

pub(super) struct Store(pub PathBuf);
impl Store {
    fn read(&self) -> Result<Catalog, String> {
        let path = self.0.join("recent.json");
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Catalog {
                    version: 1,
                    entries: vec![],
                });
            }
            Err(_) => return Err("无法读取最近工程记录，请检查目录权限".into()),
        };
        if !metadata.is_file() || metadata.len() > MAX_BYTES {
            return Err("最近工程记录不是普通文件或超过容量限制".into());
        }
        let mut bytes = vec![];
        File::open(path)
            .and_then(|file| file.take(MAX_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|_| "无法读取最近工程记录")?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("最近工程记录超过容量限制".into());
        }
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|_| "最近工程记录损坏；原工程不受影响，可通过“打开”选择文件")?;
        if catalog.version != 1 || catalog.entries.len() > MAX_ENTRIES {
            return Err("最近工程记录版本或条数不受支持".into());
        }
        let mut ids = std::collections::HashSet::new();
        let mut paths = std::collections::HashSet::new();
        for entry in &catalog.entries {
            if uuid::Uuid::parse_str(&entry.id).is_err()
                || !ids.insert(&entry.id)
                || !paths.insert(&entry.path)
                || !entry.path.is_absolute()
                || entry.path.as_os_str().len() > 4096
                || entry.name.len() > 512
                || entry.opened_at_ms > 8_640_000_000_000_000
            {
                return Err("最近工程记录内容无效；请通过“打开”选择工程".into());
            }
        }
        Ok(catalog)
    }

    pub fn list(&self) -> Result<Vec<Entry>, String> {
        Ok(self.read()?.entries)
    }

    pub fn resolve(&self, id: &str) -> Result<PathBuf, String> {
        self.list()?
            .into_iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.path)
            .ok_or_else(|| "此最近记录已移除，请刷新列表或重新选择工程文件".into())
    }

    pub fn remember(&self, path: &Path, name: &str) -> Result<(), String> {
        if !path.is_absolute() || path.as_os_str().len() > 4096 || name.len() > 512 {
            return Err("工程路径或名称过长，未加入最近记录".into());
        }
        self.update(|entries| {
            let id = entries
                .iter()
                .find(|e| e.path == path)
                .map_or_else(|| uuid::Uuid::new_v4().to_string(), |e| e.id.clone());
            entries.retain(|entry| entry.path != path);
            entries.insert(
                0,
                Entry {
                    id,
                    name: name.into(),
                    path: path.into(),
                    opened_at_ms: u64::try_from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis(),
                    )
                    .unwrap_or(8_640_000_000_000_000)
                    .min(8_640_000_000_000_000),
                },
            );
            entries.truncate(MAX_ENTRIES);
        })
    }

    pub fn forget(&self, id: &str) -> Result<(), String> {
        self.update(|entries| entries.retain(|entry| entry.id != id))
    }

    fn update(&self, change: impl FnOnce(&mut Vec<Entry>)) -> Result<(), String> {
        fs::create_dir_all(&self.0).map_err(|_| "无法创建最近工程目录")?;
        let lock_path = self.0.join("recent.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            return Err("最近工程锁文件无效".into());
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .map_err(|_| "无法锁定最近工程记录")?;
        lock.try_lock()
            .map_err(|_| "其他窗口正在更新最近工程，请稍后重试")?;
        let mut catalog = self.read()?;
        change(&mut catalog.entries);
        let bytes = serde_json::to_vec(&catalog).map_err(|_| "最近工程记录编码失败")?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("最近工程记录超过容量限制，原记录已保留".into());
        }
        let mut temporary = tempfile::Builder::new()
            .prefix(".recent-")
            .tempfile_in(&self.0)
            .map_err(|_| "无法创建最近工程临时文件")?;
        temporary
            .write_all(&bytes)
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|_| "最近工程记录写入失败，原记录已保留")?;
        temporary
            .persist(self.0.join("recent.json"))
            .map_err(|_| "无法提交最近工程记录，原记录已保留")?;
        #[cfg(unix)]
        File::open(&self.0)
            .and_then(|file| file.sync_all())
            .map_err(|_| "最近记录已更新，但目录同步未完成")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
