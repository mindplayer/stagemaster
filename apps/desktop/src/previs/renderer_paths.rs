//! Per-owner renderer folders, checked before starting either child.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct RuntimePaths {
    pub root: PathBuf,
    pub logs: PathBuf,
    pub temporary: PathBuf,
    pub user: PathBuf,
    pub cache: PathBuf,
}

impl RuntimePaths {
    pub(super) fn editor(project: PathBuf) -> Self {
        Self {
            logs: project.join("logs"),
            temporary: project.join("tmp"),
            user: project.join("data/previs-user"),
            cache: project.join("data/previs-derived-cache"),
            root: project,
        }
    }

    pub(super) fn component(owner: &crate::storage_paths::Directories) -> Self {
        let root = owner.data.join("previs");
        Self {
            logs: owner.logs.join("previs"),
            temporary: owner.temporary.join("previs"),
            user: root.join("user"),
            cache: root.join("cache"),
            root,
        }
    }

    pub(super) fn prepare(&self) -> Result<(), String> {
        let platform_user = self.user.join("platform-user");
        let paths = [
            &self.root,
            &self.logs,
            &self.temporary,
            &self.user,
            &self.cache,
            &platform_user,
        ];
        // Inspect *all* existing ancestors before any mkdir, so a late symlink does
        // not redirect an earlier directory creation into a different owner.
        for path in paths {
            plain_ancestors(path)?;
        }
        for path in paths {
            fs::create_dir_all(path).map_err(|_| "无法创建本地预演数据目录")?;
            plain_ancestors(path)?;
        }
        Ok(())
    }
}

fn plain_ancestors(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("本地预演数据目录无效或包含链接".into()),
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "renderer_paths_tests.rs"]
mod tests;
