//! Locate an installed component; package qualification remains a separate build step.
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub(super) struct Component {
    pub program: PathBuf,
    pub node: PathBuf,
    pub signalling: PathBuf,
}

fn executable(base: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        base.join("StageMasterPreview.app/Contents/MacOS/StageMasterPreview")
    } else if cfg!(target_os = "windows") {
        base.join("StageMasterPreview.exe")
    } else {
        base.join("StageMasterPreview")
    }
}

pub(super) fn installed(base: &Path) -> Result<Option<Component>, String> {
    match fs::symlink_metadata(base) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(metadata) if metadata.file_type().is_dir() => {}
        _ => return Err("三维预演组件目录无效，不能改用开发编辑器".into()),
    }
    let program = executable(base);
    let node = base.join(if cfg!(target_os = "windows") {
        "node.exe"
    } else {
        "node"
    });
    let signalling = base.join("signalling.mjs");
    for entry in [&program, &node, &signalling] {
        required_file(base, entry)?;
    }
    for entry in [&program, &node] {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(entry)
                .map_err(|_| "无法读取三维预演组件")?
                .permissions()
                .mode()
                & 0o111
                == 0
            {
                return Err("三维预演组件入口不可执行".into());
            }
        }
        #[cfg(not(unix))]
        let _ = entry;
    }
    let context = [
        "package.json",
        "package-lock.json",
        "node_modules/@epicgames-ps/lib-pixelstreamingcommon-ue5.8/package.json",
        "node_modules/@epicgames-ps/lib-pixelstreamingcommon-ue5.8/dist/cjs/pixelstreamingcommon.js",
        "node_modules/@epicgames-ps/lib-pixelstreamingsignalling-ue5.8/package.json",
        "node_modules/@epicgames-ps/lib-pixelstreamingsignalling-ue5.8/dist/cjs/pixelstreamingsignalling.js",
    ];
    for relative in context {
        let path = base.join(relative);
        required_file(base, &path)?;
        if Path::new(relative)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            let mut bytes = Vec::new();
            fs::File::open(path)
                .and_then(|file| file.take(1_048_577).read_to_end(&mut bytes))
                .map_err(|_| "无法读取三维预演组件清单")?;
            if bytes.len() > 1_048_576
                || !serde_json::from_slice::<serde_json::Value>(&bytes)
                    .is_ok_and(|value| value.is_object())
            {
                return Err(format!("三维预演组件清单无效：{relative}"));
            }
        }
    }
    if cfg!(target_os = "macos") {
        for relative in [
            "StageMasterPreview.app/Contents/Info.plist",
            "StageMasterPreview.app/Contents/UE/StageMasterPreview/Content/Paks/StageMasterPreview-Mac.pak",
        ] {
            required_file(base, &base.join(relative))?;
        }
    }
    Ok(Some(Component {
        program,
        node,
        signalling,
    }))
}

fn required_file(base: &Path, path: &Path) -> Result<(), String> {
    let relative = path
        .strip_prefix(base)
        .map_err(|_| "三维预演组件路径无效")?;
    for ancestor in path.ancestors().take_while(|path| *path != base) {
        let metadata = fs::symlink_metadata(ancestor)
            .map_err(|_| format!("三维预演组件不完整：缺少 {}", relative.display()))?;
        if metadata.file_type().is_symlink()
            || (ancestor == path && (!metadata.is_file() || metadata.len() == 0))
            || (ancestor != path && !metadata.is_dir())
        {
            return Err(format!("三维预演组件路径无效：{}", relative.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "renderer_component_tests.rs"]
mod tests;
