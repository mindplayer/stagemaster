//! Leased, atomic recovery snapshots. Source paths are labels, never save destinations.
mod discovery;
mod record;
#[cfg(test)]
mod scan_tests;
use record::{Record, digest};
use serde::Serialize;
use stagemaster_project::{Document, MAX_BYTES};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const MAX_RECOVERY_RECORDS: usize = 64;
const MAX_RECORD_BYTES: usize = MAX_BYTES + 64 * 1024;
const OLD_AFTER_MS: u64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Clone)]
pub struct RecoveryStore {
    root: PathBuf,
}
pub struct RecoverySession {
    store: RecoveryStore,
    id: String,
    lease: Option<File>,
}
pub struct RecoveryCandidate {
    store: RecoveryStore,
    id: String,
    token: String,
    lease: Option<File>,
    pub document: Document,
    pub source_file: Option<String>,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryState {
    Ready,
    Active,
    Damaged,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryEntry {
    pub id: String,
    pub token: String,
    pub project_name: Option<String>,
    pub captured_at_ms: Option<u64>,
    pub source_file: Option<String>,
    pub state: RecoveryState,
    pub older: bool,
    pub problem: Option<String>,
    pub can_discard: bool,
}
#[derive(Debug, Serialize)]
pub struct RecoveryCatalog {
    pub entries: Vec<RecoveryEntry>,
    pub omitted: usize,
}

impl RecoveryStore {
    /// The host injects a dedicated local directory, never the source project's directory.
    /// # Errors
    /// Rejects unavailable directories and symbolic links at the recovery root.
    pub fn open(root: &Path) -> Result<Self, String> {
        if fs::symlink_metadata(root).is_ok_and(|m| !m.is_dir()) {
            return Err("恢复目录不是普通目录".into());
        }
        fs::create_dir_all(root).map_err(|_| "无法创建恢复目录，请检查权限或磁盘空间")?;
        Ok(Self {
            root: root.canonicalize().map_err(|_| "恢复目录不可用")?,
        })
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Hold one OS lease for the lifetime of a window's editing session.
    /// # Errors
    /// Rejects unavailable or unwritable recovery storage.
    pub fn begin(&self) -> Result<RecoverySession, String> {
        let _index = self.index()?;
        let id = Uuid::new_v4().to_string();
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(self.lease_path(&id))
            .map_err(|_| "无法创建恢复会话")?;
        lease.try_lock().map_err(|_| "无法锁定恢复会话")?;
        Ok(RecoverySession {
            store: self.clone(),
            id,
            lease: Some(lease),
        })
    }
    /// Discover complete records, including damaged and currently active sessions.
    /// # Errors
    /// Rejects an unreadable directory/index; individual bad records remain visible.
    pub fn list(&self, now_ms: u64) -> Result<RecoveryCatalog, String> {
        let _index = self.index()?;
        self.clean_orphans()?;
        let discovered = discovery::scan(&self.root)?;
        let omitted = discovered.omitted;
        let mut entries = Vec::new();
        for id in discovered.ids {
            entries.push(self.entry(&id, now_ms));
        }
        entries.sort_by(|a, b| {
            b.captured_at_ms
                .cmp(&a.captured_at_ms)
                .then(a.id.cmp(&b.id))
        });
        Ok(RecoveryCatalog { entries, omitted })
    }
    /// Claim a stable inactive snapshot. Holding the candidate prevents a second recovery.
    /// # Errors
    /// Rejects occupied, changed, malformed and unsupported records.
    pub fn claim(&self, id: &str, token: &str) -> Result<RecoveryCandidate, String> {
        valid_id(id)?;
        let _index = self.index()?;
        let lease = self.acquire(id)?;
        let observation = self.observe(id);
        if observation.token != token {
            return Err("恢复副本已变化，请刷新后重试".into());
        }
        let bytes = observation.bytes?;
        let record = Record::decode(&bytes, id)?;
        let document = record.document()?;
        Ok(RecoveryCandidate {
            store: self.clone(),
            id: id.into(),
            token: token.into(),
            lease: Some(lease),
            document,
            source_file: record.source_file,
        })
    }
    /// Explicitly discard one unchanged, inactive record, including a damaged regular file.
    /// # Errors
    /// Rejects occupied/changed records, non-regular paths and I/O failures.
    pub fn discard(&self, id: &str, token: &str) -> Result<(), String> {
        valid_id(id)?;
        let _index = self.index()?;
        let lease = self.acquire(id)?;
        self.remove_snapshot(id, Some(token))?;
        drop(lease);
        self.remove_lease(id)
    }
    fn index(&self) -> Result<File, String> {
        let path = self.root.join("index.lock");
        let file = open_regular_lock(&path)?;
        file.lock().map_err(|_| "无法锁定恢复目录")?;
        Ok(file)
    }
    fn path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.recovery.json"))
    }
    fn lease_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.lease"))
    }
    fn ids(&self) -> Result<Vec<String>, String> {
        Ok(discovery::scan(&self.root)?.ids)
    }
    fn acquire(&self, id: &str) -> Result<File, String> {
        let lease = open_regular_lock(&self.lease_path(id))?;
        match lease.try_lock() {
            Ok(()) => Ok(lease),
            Err(TryLockError::WouldBlock) => {
                Err("此恢复副本仍由其他窗口使用，请回到该窗口继续编辑或保存".into())
            }
            Err(TryLockError::Error(_)) => Err("无法确认恢复会话是否仍在使用，已停止操作".into()),
        }
    }
    fn entry(&self, id: &str, now: u64) -> RecoveryEntry {
        let observation = self.observe(id);
        let mut entry = RecoveryEntry {
            id: id.into(),
            token: observation.token,
            project_name: None,
            captured_at_ms: None,
            source_file: None,
            state: RecoveryState::Damaged,
            older: false,
            problem: None,
            can_discard: false,
        };
        let lease = match open_regular_lock(&self.lease_path(id)) {
            Ok(lease) => lease,
            Err(error) => {
                entry.problem = Some(error);
                return entry;
            }
        };
        match lease.try_lock() {
            Ok(()) => {
                entry.can_discard = observation.regular;
            }
            Err(TryLockError::WouldBlock) => entry.state = RecoveryState::Active,
            Err(TryLockError::Error(_)) => {
                entry.problem = Some("无法确认会话占用状态".into());
                return entry;
            }
        }
        let decoded = observation
            .bytes
            .and_then(|bytes| Record::decode(&bytes, id));
        match decoded {
            Ok(record) => {
                entry.project_name = Some(record.project_name.clone());
                entry.source_file.clone_from(&record.source_file);
                entry.captured_at_ms = Some(record.captured_at_ms);
                entry.older = now.saturating_sub(record.captured_at_ms) > OLD_AFTER_MS;
                if record.captured_at_ms > now.saturating_add(5 * 60 * 1000) {
                    entry.problem = Some("恢复时间晚于本机时间，请核对系统时钟".into());
                }
                if entry.state != RecoveryState::Active {
                    match record.document() {
                        Ok(_) => entry.state = RecoveryState::Ready,
                        Err(error) => entry.problem = Some(error),
                    }
                }
            }
            Err(error) => entry.problem = Some(error),
        }
        entry
    }
    fn observe(&self, id: &str) -> Observation {
        let path = self.path(id);
        let Ok(meta) = fs::symlink_metadata(&path) else {
            return Observation {
                token: "missing".into(),
                regular: false,
                bytes: Err("恢复文件已不存在或无法读取".into()),
            };
        };
        let fallback = format!("metadata:{}:{:?}", meta.len(), meta.modified());
        if !meta.is_file() {
            return Observation {
                token: fallback,
                regular: false,
                bytes: Err("恢复文件不是普通文件，不支持符号链接或目录".into()),
            };
        }
        if meta.len() > MAX_RECORD_BYTES as u64 {
            return Observation {
                token: fallback,
                regular: true,
                bytes: Err("恢复文件超过容量限制".into()),
            };
        }
        let mut bytes = Vec::new();
        let result = File::open(&path).and_then(|file| {
            file.take((MAX_RECORD_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
        });
        if result.is_err() || bytes.len() > MAX_RECORD_BYTES {
            return Observation {
                token: fallback,
                regular: true,
                bytes: Err("恢复文件读取失败或超过容量限制".into()),
            };
        }
        Observation {
            token: digest(&bytes),
            regular: true,
            bytes: Ok(bytes),
        }
    }
    fn remove_snapshot(&self, id: &str, token: Option<&str>) -> Result<(), String> {
        let path = self.path(id);
        match fs::symlink_metadata(&path) {
            Ok(m) if m.is_file() => {
                if let Some(token) = token
                    && self.observe(id).token != token
                {
                    return Err("恢复副本已变化，请刷新后重试".into());
                }
                fs::remove_file(path).map_err(|_| "无法丢弃恢复副本，请检查权限")?;
            }
            Ok(_) => return Err("恢复位置不是普通文件，已停止清理".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("无法读取恢复文件状态".into()),
        }
        self.clean_temporary(id)?;
        super::sync_directory(&self.root).map_err(|()| "恢复副本已移除，但目录同步失败".into())
    }
    fn clean_temporary(&self, id: &str) -> Result<(), String> {
        let prefix = format!(".recovery-{id}-");
        for entry in fs::read_dir(&self.root).map_err(|_| "无法清理恢复临时文件")? {
            let entry = entry.map_err(|_| "无法读取恢复临时文件")?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&prefix)
                && name.ends_with(".tmp")
                && entry.file_type().is_ok_and(|t| t.is_file())
            {
                fs::remove_file(entry.path()).map_err(|_| "无法清理恢复临时文件")?;
            }
        }
        Ok(())
    }
    fn remove_lease(&self, id: &str) -> Result<(), String> {
        match fs::remove_file(self.lease_path(id)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("恢复文件已移除，但会话标记未清理".into()),
        }
    }
    fn missing(&self, id: &str) -> bool {
        fs::symlink_metadata(self.path(id)).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    }
    fn clean_orphans(&self) -> Result<(), String> {
        // Only abandoned leases without a payload are collectible. An old payload is
        // never garbage, and an active lease may be waiting to write its first snapshot.
        for entry in fs::read_dir(&self.root).map_err(|_| "无法读取恢复目录")? {
            let entry = entry.map_err(|_| "无法读取恢复目录项目")?;
            let name = entry.file_name();
            let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".lease")) else {
                continue;
            };
            if valid_id(id).is_ok()
                && self.missing(id)
                && let Ok(lease) = self.acquire(id)
            {
                self.clean_temporary(id)?;
                drop(lease);
                self.remove_lease(id)?;
            }
        }
        Ok(())
    }
}
struct Observation {
    token: String,
    regular: bool,
    bytes: Result<Vec<u8>, String>,
}
impl RecoverySession {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Commit an exact editing snapshot, without minting a saved project revision.
    /// # Errors
    /// A failed pre-rename write leaves the previous complete recovery record untouched.
    pub fn checkpoint(
        &mut self,
        document: &Document,
        source_file: Option<String>,
        now_ms: u64,
    ) -> Result<(), String> {
        let record = Record::new(&self.id, document, source_file, now_ms)?;
        let bytes = serde_json::to_vec(&record).map_err(|_| "恢复编码失败")?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err("恢复记录超过容量限制".into());
        }
        let _index = self.store.index()?;
        let path = self.store.path(&self.id);
        match fs::symlink_metadata(&path) {
            Ok(meta) if !meta.is_file() => return Err("恢复位置不是普通文件".into()),
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err("无法读取恢复位置".into());
            }
            _ => {}
        }
        if !path.exists() && self.store.ids()?.len() >= MAX_RECOVERY_RECORDS {
            return Err("恢复副本已达 64 份，请在恢复中心清理旧副本；现有副本未被删除".into());
        }
        let mut temp = tempfile::Builder::new()
            .prefix(&format!(".recovery-{}-", self.id))
            .suffix(".tmp")
            .tempfile_in(&self.store.root)
            .map_err(|_| "无法创建恢复临时文件")?;
        temp.write_all(&bytes)
            .map_err(|_| "恢复写入失败，请检查磁盘空间")?;
        temp.as_file().sync_all().map_err(|_| "恢复文件同步失败")?;
        temp.persist(&path)
            .map_err(|_| "无法提交恢复副本，上一个检查点保留")?;
        super::sync_directory(&self.store.root)
            .map_err(|()| "恢复副本已写入，但目录同步失败".into())
    }
    /// Retire only this session's snapshot after a successful save or explicit discard.
    /// # Errors
    /// Rejects I/O failures and unexpected file types.
    pub fn clear(&mut self) -> Result<(), String> {
        let _index = self.store.index()?;
        self.store.remove_snapshot(&self.id, None)
    }
}
impl RecoveryCandidate {
    /// Consume the original only after its recovered copy has been checkpointed or saved.
    /// # Errors
    /// Rejects changed records and cleanup failures; the caller should retain this claim.
    pub fn discard(&mut self) -> Result<(), String> {
        if self.lease.is_none() {
            return Ok(());
        }
        let _index = self.store.index()?;
        self.store.remove_snapshot(&self.id, Some(&self.token))?;
        self.lease.take();
        self.store.remove_lease(&self.id)
    }
}
impl Drop for RecoverySession {
    fn drop(&mut self) {
        // Never remove a surviving snapshot on Drop: unwinding is not an authorized discard.
        if let Ok(_index) = self.store.index() {
            self.lease.take();
            if self.store.missing(&self.id) {
                let _ = self.store.clean_temporary(&self.id);
                let _ = self.store.remove_lease(&self.id);
            }
        }
    }
}
fn valid_id(id: &str) -> Result<(), String> {
    if Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id) {
        Ok(())
    } else {
        Err("恢复项目标识无效".into())
    }
}
fn open_regular_lock(path: &Path) -> Result<File, String> {
    if fs::symlink_metadata(path).is_ok_and(|m| !m.is_file()) {
        return Err("恢复锁不是普通文件".into());
    }
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| "无法打开恢复锁，请检查目录权限".into())
}
