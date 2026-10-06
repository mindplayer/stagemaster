use super::*;
use std::{collections::BTreeMap, ffi::OsStr, fs};

fn temporary() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("previs-014-environment-")
        .tempdir_in(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tmp")
                .canonicalize()
                .unwrap(),
        )
        .unwrap()
}
fn environment(command: &Command) -> BTreeMap<&OsStr, Option<&OsStr>> {
    command.get_envs().collect()
}
fn component_paths(root: &Path) -> super::super::renderer_paths::RuntimePaths {
    super::super::renderer_paths::RuntimePaths::component(&crate::storage_paths::Directories {
        data: root.join("data"),
        temporary: root.join("tmp"),
        logs: root.join("logs"),
    })
}
fn foreign_environment(command: &mut Command) {
    command
        .env("CFFIXED_USER_HOME", "/foreign-user")
        .env("DYLD_LIBRARY_PATH", "/foreign/lib")
        .env("DYLD_INSERT_LIBRARIES", "/foreign/inject.dylib")
        .env("DYLD_FRAMEWORK_PATH", "/foreign/framework")
        .env("DYLD_FALLBACK_LIBRARY_PATH", "/foreign/fallback");
}
fn assert_owner_environment(command: &Command, paths: &super::super::renderer_paths::RuntimePaths) {
    let env = environment(command);
    assert_eq!(
        env[OsStr::new("CFFIXED_USER_HOME")],
        Some(paths.user.join("platform-user").as_os_str())
    );
    for key in [
        "DYLD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
    ] {
        assert_eq!(
            env[OsStr::new(key)],
            None,
            "{key} not inherited or explicitly retained"
        );
    }
    assert!(!env.contains_key(OsStr::new("HOME")));
    assert_eq!(env[OsStr::new("TMPDIR")], Some(paths.temporary.as_os_str()));
    assert_eq!(
        env[OsStr::new("UE_LocalDataCachePath")],
        Some(paths.cache.as_os_str())
    );
}

#[test]
fn editor_foreign_home_and_loader_are_overridden_without_losing_prebuilt_role() {
    let temp = temporary();
    let paths = super::super::renderer_paths::RuntimePaths::editor(temp.path().to_owned());
    let mut command = Command::new("not-launched-editor");
    foreign_environment(&mut command);
    configure_editor_runtime(&mut command);
    configure_runtime(&mut command, &paths).unwrap();
    assert_owner_environment(&command, &paths);
    assert_eq!(
        environment(&command)[OsStr::new("UE_SKIP_UBT_SDK_SETUP")],
        Some(OsStr::new("1"))
    );
    assert_eq!(command.get_args().count(), 3);
}

#[test]
fn component_explicit_loader_overrides_are_removed_and_sdk_shortcut_stays_removed() {
    let temp = temporary();
    let paths = component_paths(temp.path());
    let mut command = Command::new("not-launched-game");
    foreign_environment(&mut command);
    command.env("UE_SKIP_UBT_SDK_SETUP", "1");
    configure_component_runtime(&mut command, &paths).unwrap();
    configure_runtime(&mut command, &paths).unwrap();
    assert_owner_environment(&command, &paths);
    assert_eq!(
        environment(&command)[OsStr::new("UE_SKIP_UBT_SDK_SETUP")],
        None
    );
}

#[test]
fn validation_failure_preserves_the_original_command_without_new_directories() {
    let temp = temporary();
    let root = temp.path().join("not-a-directory");
    fs::write(&root, b"keep").unwrap();
    let paths = super::super::renderer_paths::RuntimePaths::editor(root.clone());
    let mut command = Command::new("not-launched");
    foreign_environment(&mut command);
    let before: Vec<_> = command
        .get_envs()
        .map(|(key, value)| (key.to_owned(), value.map(ToOwned::to_owned)))
        .collect();
    assert!(configure_runtime(&mut command, &paths).is_err());
    let after: Vec<_> = command
        .get_envs()
        .map(|(key, value)| (key.to_owned(), value.map(ToOwned::to_owned)))
        .collect();
    assert_eq!(before, after);
    assert_eq!(command.get_args().count(), 0);
    assert!(command.get_current_dir().is_none());
    assert_eq!(fs::read(root).unwrap(), b"keep");
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn owners_do_not_share_foundation_home_or_migrate_old_user_data() {
    let temp = temporary();
    let old = temp.path().join("old-platform-user");
    fs::create_dir(&old).unwrap();
    fs::write(old.join("sentinel"), b"original").unwrap();
    let mut homes = Vec::new();
    for name in ["owner-a", "owner-b"] {
        let paths = super::super::renderer_paths::RuntimePaths::editor(temp.path().join(name));
        let mut command = Command::new("not-launched");
        command.env("CFFIXED_USER_HOME", &old);
        configure_runtime(&mut command, &paths).unwrap();
        assert_eq!(
            environment(&command)[OsStr::new("CFFIXED_USER_HOME")],
            Some(paths.user.join("platform-user").as_os_str())
        );
        homes.push(paths.user.join("platform-user"));
        assert_eq!(fs::read(old.join("sentinel")).unwrap(), b"original");
        assert_eq!(fs::read_dir(&old).unwrap().count(), 1);
    }
    assert_ne!(homes[0], homes[1]);
}

#[cfg(target_os = "macos")]
#[test]
fn native_foundation_resolves_the_configured_owner_instead_of_the_global_user() {
    for component in [false, true] {
        let temp = temporary();
        let paths = if component {
            component_paths(temp.path())
        } else {
            super::super::renderer_paths::RuntimePaths::editor(temp.path().to_owned())
        };
        let mut command = Command::new("/usr/bin/osascript");
        let script = r"ObjC.import('Foundation'); function run(argv) { return JSON.stringify({home:$.NSHomeDirectory().js, support:$.NSFileManager.defaultManager.URLsForDirectoryInDomains(14,1).firstObject.path.js, cache:$.NSFileManager.defaultManager.URLsForDirectoryInDomains(13,1).firstObject.path.js, arguments:argv}); }";
        command.args(["-l", "JavaScript", "-e", script, "--"]);
        command.env("CFFIXED_USER_HOME", temp.path().join("old-user"));
        if component {
            configure_component_runtime(&mut command, &paths).unwrap();
        } else {
            configure_editor_runtime(&mut command);
        }
        configure_runtime(&mut command, &paths).unwrap();
        let home = paths.user.join("platform-user");
        // Never launch the pre-repair command with a foreign/global Foundation home.
        assert_eq!(
            environment(&command)[OsStr::new("CFFIXED_USER_HOME")],
            Some(home.as_os_str())
        );
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["home"], home.to_str().unwrap());
        assert_eq!(
            value["support"],
            home.join("Library/Application Support").to_str().unwrap()
        );
        assert_eq!(
            value["cache"],
            home.join("Library/Caches").to_str().unwrap()
        );
        let arguments: Vec<_> = command
            .get_args()
            .skip(5)
            .map(|arg| arg.to_str().unwrap())
            .collect();
        assert_eq!(value["arguments"], serde_json::json!(arguments));
        value["role"] = serde_json::json!(if component { "component" } else { "editor" });
        println!("PREVIS014_FOUNDATION {value}");
    }
}

#[test]
fn loader_cleanup_keeps_home_path_and_unrelated_business_environment() {
    let mut command = Command::new("not-launched");
    command
        .env("HOME", "/unchanged-home")
        .env("PATH", "/unchanged-path")
        .env("STAGEMASTER_STREAM_URL", "original-stream-marker")
        .env("DYLD", "unchanged-non-loader-name")
        .env("DYLD_FRAMEWORK_PATH", "/foreign");
    clear_loader_overrides(&mut command);
    let env = environment(&command);
    assert_eq!(env[OsStr::new("HOME")], Some(OsStr::new("/unchanged-home")));
    assert_eq!(env[OsStr::new("PATH")], Some(OsStr::new("/unchanged-path")));
    assert_eq!(
        env[OsStr::new("STAGEMASTER_STREAM_URL")],
        Some(OsStr::new("original-stream-marker"))
    );
    assert_eq!(
        env[OsStr::new("DYLD")],
        Some(OsStr::new("unchanged-non-loader-name"))
    );
    assert_eq!(env[OsStr::new("DYLD_FRAMEWORK_PATH")], None);
}

#[cfg(unix)]
#[test]
fn non_unicode_loader_keys_are_still_removed_by_their_byte_prefix() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let key = OsString::from_vec(b"DYLD_\xff".to_vec());
    let mut command = Command::new("not-launched");
    command.env(&key, "/foreign");
    clear_loader_overrides(&mut command);
    assert_eq!(environment(&command)[key.as_os_str()], None);
}
