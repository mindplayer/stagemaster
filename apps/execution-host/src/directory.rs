use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// A new, private run directory. Existing directories are never reused or cleared.
pub(crate) struct Directory {
    root: PathBuf,
    published: bool,
}
impl Directory {
    pub fn create(path: &Path) -> io::Result<Self> {
        // The current executable is deliberately limited to tested POSIX permissions.
        #[cfg(not(unix))]
        return Err(io::Error::other("此宿主尚未验证当前系统的私有目录权限"));
        #[cfg(unix)]
        {
            use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt};
            DirBuilder::new().mode(0o700).create(path)?;
            Ok(Self {
                root: path.canonicalize()?,
                published: false,
            })
        }
    }
    pub fn store(&self) -> PathBuf {
        self.root.join("store")
    }
    pub fn publish(&mut self, value: &impl Serialize) -> io::Result<()> {
        let temporary = self.root.join(".discovery.tmp");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(value)?)?;
        file.sync_all()?;
        fs::rename(temporary, self.root.join("discovery.json"))?;
        self.published = true;
        fs::File::open(&self.root)?.sync_all()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.root.join(".discovery.tmp"));
        if self.published {
            let _ = fs::remove_file(self.root.join("discovery.json"));
        }
    }
}
