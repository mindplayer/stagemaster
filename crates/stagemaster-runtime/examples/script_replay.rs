//! Source-to-installed script acceptance only; no sound, renderer or physical output.
#[path = "script_replay/drive.rs"]
mod drive;
#[path = "script_replay/prepare.rs"]
mod prepare;
#[path = "script_replay/schedule.rs"]
mod schedule;
#[path = "script_replay/store.rs"]
mod store;
use std::{
    error::Error,
    fs::File,
    io::Read,
    path::{Component, Path},
};

fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute
        .components()
        .any(|c| matches!(c, Component::ParentDir))
        || !absolute.starts_with(&root)
    {
        return Err("验收输入必须使用项目内不含父目录跳转的路径".into());
    }
    if absolute.starts_with(root.join("output")) {
        return Err("不读取用户 output/".into());
    }
    let mut checked = root.clone();
    for component in absolute.strip_prefix(&root)?.components() {
        checked.push(component);
        if std::fs::symlink_metadata(&checked)?
            .file_type()
            .is_symlink()
        {
            return Err("验收输入及祖先目录不得是符号链接".into());
        }
    }
    if !std::fs::symlink_metadata(&absolute)?.is_file() {
        return Err("验收输入必须是普通文件".into());
    }
    let mut bytes = Vec::new();
    File::open(absolute)?
        .take(u64::try_from(maximum)? + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err("验收输入超出容量".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> tempfile::TempDir {
        tempfile::tempdir_in(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tmp")
                .canonicalize()
                .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn bounded_reader_refuses_outside_parent_and_user_output_before_opening() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        for path in [
            root.join("output/not-read.json"),
            root.join("data/../output/not-read.json"),
            Path::new("/etc/hosts").to_path_buf(),
        ] {
            assert!(read(&path, 10).is_err());
        }
        let dir = directory();
        let file = dir.path().join("input.json");
        std::fs::write(&file, b"12345").unwrap();
        assert_eq!(read(&file, 5).unwrap(), b"12345");
        assert!(read(&file, 4).unwrap_err().to_string().contains("容量"));
        assert!(
            read(dir.path(), 5)
                .unwrap_err()
                .to_string()
                .contains("普通文件")
        );
    }
    #[test]
    #[cfg(unix)]
    fn reader_refuses_a_link_in_any_input_directory_component() {
        let dir = directory();
        std::fs::create_dir(dir.path().join("original")).unwrap();
        std::fs::write(dir.path().join("original/input.json"), b"{}").unwrap();
        std::os::unix::fs::symlink(dir.path().join("original"), dir.path().join("alias")).unwrap();
        assert!(
            read(&dir.path().join("alias/input.json"), 10)
                .unwrap_err()
                .to_string()
                .contains("符号链接")
        );
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("用法：script_replay 源工程.json 原播放包.smpkg 场景列表ID；仅软件核验".into());
    }
    let source = read(Path::new(&args[0]), stagemaster_project::MAX_BYTES)?;
    let bytes = read(Path::new(&args[1]), stagemaster_package::MAX_PACKAGE_BYTES)?;
    let id = args[2].to_str().ok_or("场景列表ID无效")?;
    let document = stagemaster_project::Document::decode(&source)?;
    let mut prepared = prepare::prepare(&document, &bytes, id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&drive::run(&mut prepared)?)?
    );
    Ok(())
}
