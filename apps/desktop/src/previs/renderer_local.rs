//! Process-local UE runtime paths; never change the user's global engine preferences.
use std::{path::Path, process::Command};

pub(super) fn configure_signalling(
    command: &mut Command,
    paths: &super::renderer_paths::RuntimePaths,
) {
    for (key, _) in std::env::vars_os() {
        if key
            .to_str()
            .is_some_and(|key| key.starts_with("NODE_") || key.starts_with("DYLD_"))
        {
            command.env_remove(key);
        }
    }
    configure_signalling_paths(command, paths);
}

fn configure_signalling_paths(command: &mut Command, paths: &super::renderer_paths::RuntimePaths) {
    command
        .current_dir(&paths.root)
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .env_remove("NODE_COMPILE_CACHE")
        .env("NODE_DISABLE_COMPILE_CACHE", "1")
        .env("TMPDIR", &paths.temporary);
}

#[cfg(test)]
fn configure(command: &mut Command, root: &Path) -> Result<(), String> {
    configure_runtime(
        command,
        &super::renderer_paths::RuntimePaths::editor(root.to_owned()),
    )
}

pub(super) fn configure_runtime(
    command: &mut Command,
    paths: &super::renderer_paths::RuntimePaths,
) -> Result<(), String> {
    paths.prepare()?;
    configure_paths(
        command,
        &paths.root,
        &paths.temporary,
        &paths.user,
        &paths.cache,
    );
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

pub(super) fn configure_component_runtime(
    command: &mut Command,
    paths: &super::renderer_paths::RuntimePaths,
) -> Result<(), String> {
    paths.prepare()?;
    for (key, _) in std::env::vars_os() {
        if key.to_str().is_some_and(|key| key.starts_with("DYLD_")) {
            command.env_remove(key);
        }
    }
    configure_component_user(command, &paths.user);
    Ok(())
}

fn configure_component_user(command: &mut Command, user: &Path) {
    // Foundation paths are separate from UE's -UserDir. Keep both process-local.
    command
        .env("CFFIXED_USER_HOME", user.join("platform-user"))
        .env_remove("DYLD_LIBRARY_PATH")
        .env_remove("DYLD_INSERT_LIBRARIES")
        .env_remove("UE_SKIP_UBT_SDK_SETUP");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn signalling_cannot_use_a_global_loader_or_compile_cache_directory() {
        let paths = super::super::renderer_paths::RuntimePaths::editor(
            Path::new(env!("CARGO_MANIFEST_DIR")).to_owned(),
        );
        let mut command = Command::new("node");
        command
            .env("NODE_OPTIONS", "--require=outside.cjs")
            .env("NODE_PATH", "outside")
            .env("NODE_COMPILE_CACHE", "outside");
        configure_signalling_paths(&mut command, &paths);
        assert_eq!(command.get_current_dir(), Some(paths.root.as_path()));
        let env: std::collections::BTreeMap<_, _> = command.get_envs().collect();
        assert_eq!(env[OsStr::new("NODE_OPTIONS")], None);
        assert_eq!(env[OsStr::new("NODE_PATH")], None);
        assert_eq!(env[OsStr::new("NODE_COMPILE_CACHE")], None);
        assert_eq!(
            env[OsStr::new("NODE_DISABLE_COMPILE_CACHE")],
            Some(OsStr::new("1"))
        );
        assert_eq!(env[OsStr::new("TMPDIR")], Some(paths.temporary.as_os_str()));
        assert_eq!(command.get_args().count(), 0);
    }

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
            "本地预演数据目录无效或包含链接"
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

    #[test]
    fn packaged_platform_user_does_not_inherit_a_foreign_home_or_editor_shortcut() {
        let user = Path::new("/project/tmp/desktop-a/previs/user");
        let mut command = Command::new("game");
        command
            .env("CFFIXED_USER_HOME", "/outside")
            .env("DYLD_LIBRARY_PATH", "/outside")
            .env("DYLD_INSERT_LIBRARIES", "/outside/lib.dylib")
            .env("UE_SKIP_UBT_SDK_SETUP", "1");
        configure_component_user(&mut command, user);
        let env: std::collections::BTreeMap<_, _> = command.get_envs().collect();
        assert_eq!(
            env[OsStr::new("CFFIXED_USER_HOME")],
            Some(user.join("platform-user").as_os_str())
        );
        for key in [
            "DYLD_LIBRARY_PATH",
            "DYLD_INSERT_LIBRARIES",
            "UE_SKIP_UBT_SDK_SETUP",
        ] {
            assert_eq!(env[OsStr::new(key)], None);
        }
        assert!(!env.contains_key(OsStr::new("HOME")));
        assert_eq!(command.get_args().count(), 0);
    }
}
