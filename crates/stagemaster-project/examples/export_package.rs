//! Explicit file-only export through the same compiler as the desktop UI.
use stagemaster_project::{Document, MAX_BYTES, PackageSelection};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

fn read(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(u64::try_from(MAX_BYTES)? + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err("输入文件超过工程容量".into());
    }
    Ok(bytes)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("用法：export_package 工程.json 节目选择.json 新文件.smpkg；不连接设备".into());
    }
    let document = Document::decode(&read(Path::new(&args[0]))?)?;
    let selection: Vec<PackageSelection> = serde_json::from_slice(&read(Path::new(&args[1]))?)?;
    let build = document.build_package(&selection).map_err(|issues| {
        serde_json::to_string(&issues).unwrap_or_else(|_| "节目编译失败".into())
    })?;
    // Do not truncate or replace an existing user file, even when the compiler fails.
    let mut output = File::options()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.write_all(&build.bytes)?;
    output.sync_all()?;
    println!("{}", serde_json::to_string_pretty(&build.report)?);
    Ok(())
}
