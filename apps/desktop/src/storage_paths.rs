//! One desktop storage owner; development and acceptance never use the user's default data.
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tauri::Manager;

pub(crate) struct Directories {
    pub data: PathBuf,
    pub temporary: PathBuf,
    pub logs: PathBuf,
}

pub(crate) fn directories(app: &impl Manager<tauri::Wry>) -> Result<Directories, tauri::Error> {
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if let Some(selected) = isolated(
        &project,
        std::env::var_os("STAGEMASTER_ACCEPTANCE_INSTANCE").as_deref(),
        &app.app_handle().config().identifier,
        cfg!(debug_assertions),
        cfg!(feature = "internal-acceptance"),
    )? {
        Ok(selected)
    } else {
        let data = app.path().app_local_data_dir()?;
        Ok(Directories {
            temporary: data.join("tmp"),
            logs: data.join("logs"),
            data,
        })
    }
}

fn development(project: &Path, instance: Option<&OsStr>) -> Result<Directories, std::io::Error> {
    if let Some(instance) = instance {
        let name = instance
            .to_str()
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 64
                    && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
            .ok_or_else(|| std::io::Error::other("验收实例名称无效"))?;
        let data = project.join("tmp").join(format!("desktop-{name}"));
        Ok(Directories {
            temporary: data.join("tmp"),
            logs: data.join("logs"),
            data,
        })
    } else {
        Ok(Directories {
            data: project.join("data"),
            temporary: project.join("tmp"),
            logs: project.join("logs"),
        })
    }
}

fn isolated(
    project: &Path,
    instance: Option<&OsStr>,
    identifier: &str,
    debug: bool,
    internal_acceptance: bool,
) -> Result<Option<Directories>, std::io::Error> {
    if debug {
        return development(project, instance).map(Some);
    }
    if !internal_acceptance {
        return Ok(None);
    }
    if instance.is_none() && !identifier.starts_with("cn.stagemaster.acceptance.") {
        return Ok(None);
    }
    let name = instance
        .and_then(OsStr::to_str)
        .ok_or_else(|| std::io::Error::other("内部发布验收必须指定有效实例"))?;
    let selected = development(project, instance)?;
    if identifier != format!("cn.stagemaster.acceptance.{}", name.to_ascii_lowercase()) {
        return Err(std::io::Error::other("内部发布验收身份与实例不匹配"));
    }
    Ok(Some(selected))
}

#[cfg(test)]
#[path = "storage_paths_release_tests.rs"]
mod release_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_and_acceptance_roots_keep_the_existing_storage_locations() {
        let project = Path::new("project");
        let defaults = development(project, None).unwrap();
        assert_eq!(
            defaults.data.join("recovery"),
            project.join("data/recovery")
        );
        assert_eq!(defaults.logs, project.join("logs"));
        assert_eq!(defaults.temporary, project.join("tmp"));
        let a = development(project, Some(OsStr::new("previs-006-a"))).unwrap();
        let b = development(project, Some(OsStr::new("previs-006-b"))).unwrap();
        assert_eq!(
            a.data.join("recovery"),
            project.join("tmp/desktop-previs-006-a/recovery")
        );
        assert_ne!(a.data, b.data);
        assert!(a.logs.starts_with(&a.data) && a.temporary.starts_with(&a.data));
    }

    #[test]
    fn invalid_instance_names_cannot_escape_or_silently_use_default_data() {
        for name in ["", ".", "../other", "a/b", "有空格", "a b", &"a".repeat(65)] {
            assert_eq!(
                development(Path::new("project"), Some(OsStr::new(name)))
                    .err()
                    .unwrap()
                    .to_string(),
                "验收实例名称无效"
            );
        }
        assert!(development(Path::new("project"), Some(OsStr::new(&"a".repeat(64)))).is_ok());
    }
}
