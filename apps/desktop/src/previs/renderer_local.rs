//! Process-local UE runtime paths; never change the user's global engine preferences.
use std::{path::Path, process::Command};

pub(super) fn configure(command: &mut Command, root: &Path) -> Result<(), String> {
    let temporary = root.join("tmp");
    let user = root.join("data/previs-user");
    let cache = root.join("data/previs-derived-cache");
    for directory in [&temporary, &user, &cache] {
        std::fs::create_dir_all(directory).map_err(|_| "无法创建本地预演数据目录")?;
    }
    configure_paths(command, root, &temporary, &user, &cache);
    Ok(())
}

fn configure_paths(
    command: &mut Command,
    root: &Path,
    temporary: &Path,
    user: &Path,
    cache: &Path,
) {
    command
        .current_dir(root)
        .env("TMPDIR", temporary)
        // Apple/Unix UE translates '-' to '_' before getenv; the old keys were ignored.
        .env("UE_LocalDataCachePath", cache)
        .arg(format!("-LocalDataCachePath={}", cache.display()))
        .arg(format!("-UserDir={}", user.display()))
        // Reuse engine read-only packs and the local filesystem cache, not a global Zen service.
        .arg("-DDC=(ProjectPak,InstalledProjectPak,EnginePak=InstalledEnginePak,Local)");
}

pub(super) fn configure_editor_runtime(command: &mut Command) {
    // A prebuilt -game renderer does not build/export targets. Do not spawn UBT's global logs.
    // SDK/toolchain qualification remains enabled in the separate UE build commands.
    command.env("UE_SKIP_UBT_SDK_SETUP", "1");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn runtime_paths_and_cache_graph_are_process_local() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut command = Command::new("renderer");
        configure_paths(
            &mut command,
            root,
            &root.join("tmp"),
            &root.join("data/previs-user"),
            &root.join("data/previs-derived-cache"),
        );
        assert_eq!(command.get_current_dir(), Some(root));
        let environments: Vec<_> = command.get_envs().collect();
        assert_eq!(environments.len(), 2);
        for (key, value) in environments {
            assert!(matches!(
                key.to_str(),
                Some("TMPDIR" | "UE_LocalDataCachePath")
            ));
            assert!(Path::new(value.unwrap()).starts_with(root));
        }
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(arguments.len(), 3);
        assert_eq!(
            arguments[2],
            OsStr::new("-DDC=(ProjectPak,InstalledProjectPak,EnginePak=InstalledEnginePak,Local)")
        );
        for argument in &arguments[..2] {
            let (_, path) = argument.to_str().unwrap().split_once('=').unwrap();
            assert!(Path::new(path).starts_with(root));
        }
        assert!(
            !command
                .get_envs()
                .any(|(key, _)| key == "UE_SKIP_UBT_SDK_SETUP")
        );
    }

    #[test]
    fn unavailable_local_directory_does_not_configure_or_launch_the_process() {
        let mut command = Command::new("renderer");
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert_eq!(
            configure(&mut command, &file).unwrap_err(),
            "无法创建本地预演数据目录"
        );
        assert_eq!(command.get_envs().count(), 0);
        assert_eq!(command.get_args().count(), 0);
        assert!(command.get_current_dir().is_none());
    }

    #[test]
    fn only_prebuilt_editor_runtime_skips_background_sdk_export() {
        let mut command = Command::new("renderer");
        configure_editor_runtime(&mut command);
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [(OsStr::new("UE_SKIP_UBT_SDK_SETUP"), Some(OsStr::new("1")))]
        );
        assert_eq!(command.get_args().count(), 0);
    }
}
