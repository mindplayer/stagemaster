//! Exclusive, two-slot filesystem reference adapter. This is not a flash driver.
use stagemaster_install::{Commit, RECORD_BYTES, Record, Slot, Storage};
use stagemaster_package::{MAX_PACKAGE_BYTES, ReadAt};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tempfile::NamedTempFile;

pub struct FileStore {
    root: PathBuf,
    _writer: File,
    stage: Option<Stage>,
    #[cfg(test)]
    fault: std::rc::Rc<std::cell::RefCell<Fault>>,
}
struct Stage {
    slot: Slot,
    bytes: usize,
    _lease: File,
    temp: Option<NamedTempFile>,
}
impl FileStore {
    /// Open a dedicated install directory, holding its writer lock until this store is dropped.
    /// # Errors
    /// Refuse symbolic links, non-regular managed files and competing writers.
    pub fn open(root: &Path) -> io::Result<Self> {
        fs::create_dir_all(root)?;
        let meta = fs::symlink_metadata(root)?;
        if !meta.is_dir() {
            return Err(io::Error::other("安装目录不能是符号链接或非目录"));
        }
        let root = root.canonicalize()?;
        let writer = lock_file(&root.join("installer.lock"))?;
        writer
            .try_lock()
            .map_err(|_| io::Error::other("此安装目录正在被另一个进程使用"))?;
        // Only this module's interrupted temporary writes in its dedicated directory.
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if (name.starts_with(".install-payload-") || name.starts_with(".install-record-"))
                && name.ends_with(".tmp")
                && entry.file_type()?.is_file()
            {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(Self {
            root,
            _writer: writer,
            stage: None,
            #[cfg(test)]
            fault: std::rc::Rc::default(),
        })
    }
    fn payload(&self, slot: Slot) -> PathBuf {
        self.root.join(format!("slot-{}.smpkg", slot.index()))
    }
    fn record_path(&self, slot: Slot) -> PathBuf {
        self.root.join(format!("slot-{}.commit", slot.index()))
    }
    fn pin(&self, slot: Slot) -> io::Result<File> {
        lock_file(&self.root.join(format!("slot-{}.lease", slot.index())))
    }
    fn read_path(&self, slot: Slot) -> PathBuf {
        self.stage
            .as_ref()
            .filter(|s| s.slot == slot)
            .and_then(|s| s.temp.as_ref())
            .map_or_else(|| self.payload(slot), |temp| temp.path().to_path_buf())
    }
    // The same production I/O path has fallible checkpoints only in fault tests.
    #[cfg_attr(not(test), allow(clippy::unused_self, clippy::unnecessary_wraps))]
    fn point(&mut self, name: &'static str) -> io::Result<()> {
        #[cfg(test)]
        {
            let mut fault = self.fault.borrow_mut();
            fault.seen.push(name);
            if fault.exit_at == Some(fault.seen.len()) {
                std::process::exit(91);
            }
            if fault.fail_at == Some(fault.seen.len()) {
                return Err(io::Error::other(format!("注入失败：{name}")));
            }
        }
        #[cfg(not(test))]
        let _ = name;
        Ok(())
    }
    fn sync_directory(&self) -> io::Result<()> {
        File::open(&self.root)?.sync_all()
    }
}
fn regular_or_missing(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.is_file() {
                return Err(io::Error::other("受管文件不能是符号链接或非普通文件"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if meta.nlink() != 1 {
                    return Err(io::Error::other("受管文件不能具有其他硬链接"));
                }
            }
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
fn regular_file(path: &Path) -> io::Result<File> {
    if !regular_or_missing(path)? {
        return Err(io::Error::new(io::ErrorKind::NotFound, "安装载荷不存在"));
    }
    File::open(path)
}
fn lock_file(path: &Path) -> io::Result<File> {
    regular_or_missing(path)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}
fn read_at(mut file: &File, offset: usize, target: &mut [u8]) -> io::Result<()> {
    file.seek(SeekFrom::Start(
        u64::try_from(offset).map_err(|_| io::Error::other("读取偏移越界"))?,
    ))?;
    file.read_exact(target)
}
impl Storage for FileStore {
    type Error = io::Error;
    type Snapshot = FileSnapshot;
    fn capacity(&self, _slot: Slot) -> usize {
        MAX_PACKAGE_BYTES
    }
    fn record(&self, slot: Slot) -> io::Result<Record> {
        let path = self.record_path(slot);
        if !regular_or_missing(&path)? {
            return Ok(Record::Absent);
        }
        let mut file = File::open(path)?;
        if file.metadata()?.len() != RECORD_BYTES as u64 {
            return Ok(Record::Invalid);
        }
        let mut bytes = [0; RECORD_BYTES];
        file.read_exact(&mut bytes)?;
        Ok(Record::Bytes(bytes))
    }
    fn slot_len(&self, slot: Slot) -> io::Result<usize> {
        usize::try_from(regular_file(&self.read_path(slot))?.metadata()?.len())
            .map_err(|_| io::Error::other("载荷大小超限"))
    }
    fn read(&self, slot: Slot, offset: usize, target: &mut [u8]) -> io::Result<()> {
        read_at(&regular_file(&self.read_path(slot))?, offset, target)
    }
    fn prepare(&mut self, slot: Slot, bytes: usize) -> io::Result<()> {
        if self.stage.is_some() || bytes > MAX_PACKAGE_BYTES {
            return Err(io::Error::other("暂存忙或容量超限"));
        }
        self.point("prepare:before")?;
        let lease = self.pin(slot)?;
        lease
            .try_lock()
            .map_err(|_| io::Error::other("备用槽仍被运行读源占用，请先释放该读源"))?;
        self.point("prepare:lease")?;
        regular_or_missing(&self.payload(slot))?;
        let temp = tempfile::Builder::new()
            .prefix(".install-payload-")
            .suffix(".tmp")
            .tempfile_in(&self.root)?;
        self.stage = Some(Stage {
            slot,
            bytes,
            _lease: lease,
            temp: Some(temp),
        });
        self.point("prepare:created")
    }
    fn write(&mut self, slot: Slot, offset: usize, bytes: &[u8]) -> io::Result<()> {
        self.point("write:before")?;
        let stage = self
            .stage
            .as_mut()
            .filter(|s| s.slot == slot)
            .ok_or_else(|| io::Error::other("没有对应暂存事务"))?;
        if offset
            .checked_add(bytes.len())
            .is_none_or(|end| end > stage.bytes)
        {
            return Err(io::Error::other("写入超出声明的载荷范围"));
        }
        let file = stage
            .temp
            .as_mut()
            .ok_or_else(|| io::Error::other("载荷已经封存"))?
            .as_file_mut();
        file.seek(SeekFrom::Start(
            u64::try_from(offset).map_err(|_| io::Error::other("写入偏移越界"))?,
        ))?;
        #[cfg(test)]
        if self.fault.borrow().partial_write {
            file.write_all(&bytes[..bytes.len().div_ceil(2)])?;
            return Err(io::Error::other("注入部分写入失败"));
        }
        file.write_all(bytes)?;
        self.point("write:after")
    }
    fn sync_payload(&mut self, slot: Slot) -> io::Result<()> {
        self.point("payload:before-sync")?;
        let stage = self
            .stage
            .as_ref()
            .filter(|s| s.slot == slot)
            .ok_or_else(|| io::Error::other("没有对应暂存事务"))?;
        if let Some(temp) = &stage.temp {
            if temp.as_file().metadata()?.len() != stage.bytes as u64 {
                return Err(io::Error::other("暂存载荷长度不完整"));
            }
            temp.as_file().sync_all()?;
        }
        self.point("payload:after-sync")?;
        let destination = self.payload(slot);
        regular_or_missing(&destination)?;
        let stage = self
            .stage
            .as_mut()
            .ok_or_else(|| io::Error::other("暂存事务已不存在"))?;
        if let Some(temp) = stage.temp.take() {
            match temp.persist(destination) {
                Ok(_) => {}
                Err(error) => {
                    stage.temp = Some(error.file);
                    return Err(error.error);
                }
            }
        }
        self.point("payload:after-rename")?;
        self.sync_directory()?;
        self.point("payload:after-directory-sync")
    }
    fn commit_record(&mut self, commit: Commit) -> io::Result<()> {
        if self.stage.as_ref().is_none_or(|s| {
            s.slot != commit.slot || s.temp.is_some() || s.bytes != commit.identity.bytes
        }) {
            return Err(io::Error::other("载荷尚未封存"));
        }
        let bytes = commit.encode().map_err(io::Error::other)?;
        self.point("record:before")?;
        let mut temp = tempfile::Builder::new()
            .prefix(".install-record-")
            .suffix(".tmp")
            .tempfile_in(&self.root)?;
        self.point("record:created")?;
        temp.write_all(&bytes)?;
        self.point("record:written")?;
        temp.as_file().sync_all()?;
        self.point("record:synced")?;
        let destination = self.record_path(commit.slot);
        regular_or_missing(&destination)?;
        temp.persist(destination).map_err(|e| e.error)?;
        self.point("record:renamed")?;
        self.sync_directory()?;
        self.point("record:directory-synced")
    }
    fn settle(&mut self) -> io::Result<()> {
        self.point("settle:before")?;
        self.sync_directory()?;
        self.point("settle:after")
    }
    fn release(&mut self) {
        self.stage = None;
    }
    fn snapshot(&self, slot: Slot) -> io::Result<FileSnapshot> {
        let lease = self.pin(slot)?;
        lease
            .try_lock_shared()
            .map_err(|_| io::Error::other("此槽正在接收新包"))?;
        let file = regular_file(&self.payload(slot))?;
        let length = usize::try_from(file.metadata()?.len())
            .map_err(|_| io::Error::other("载荷大小超限"))?;
        Ok(FileSnapshot {
            file: Mutex::new(file),
            _lease: lease,
            length,
        })
    }
}
pub struct FileSnapshot {
    file: Mutex<File>,
    _lease: File,
    length: usize,
}
impl ReadAt for FileSnapshot {
    fn len(&self) -> usize {
        self.length
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        let file = self
            .file
            .lock()
            .map_err(|_| stagemaster_package::Error::Read)?;
        if offset
            .checked_add(target.len())
            .is_none_or(|end| end > self.length)
        {
            return Err(stagemaster_package::Error::Read);
        }
        read_at(&file, offset, target).map_err(|_| stagemaster_package::Error::Read)
    }
}

#[cfg(test)]
#[derive(Default)]
struct Fault {
    fail_at: Option<usize>,
    exit_at: Option<usize>,
    seen: Vec<&'static str>,
    partial_write: bool,
}
#[cfg(test)]
mod tests;
