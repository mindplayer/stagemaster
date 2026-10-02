//! Local software execution process. No physical output driver is linked.
#![forbid(unsafe_code)]
mod application;
mod bounded_io;
mod commands;
mod directory;
mod group;
mod preparation;
mod projection;
mod server;
mod service;
mod sessions;
mod wire;

fn main() {
    if let Err(message) = run() {
        eprintln!("执行宿主启动／运行失败：{message}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 5 || args[4] != "--software-output" {
        return Err("用法：stagemaster-execution-host 工程路径 scene|sequence|group 节目ID或来源清单路径 新运行目录 --software-output".into());
    }
    let id = args[2].to_str().ok_or("节目标识无效")?.to_owned();
    let selection = match args[1].to_str() {
        Some("scene") => Some(stagemaster_project::PackageSelection::Scene { id }),
        Some("sequence") => Some(stagemaster_project::PackageSelection::Sequence { id }),
        Some("group") => None,
        _ => return Err("节目类型须为 scene、sequence 或 group".into()),
    };
    let directory =
        directory::Directory::create(std::path::Path::new(&args[3])).map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(8)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    if let Some(selection) = selection {
        let prepared =
            preparation::prepare(std::path::Path::new(&args[0]), &selection, &directory)?;
        runtime.block_on(server::serve(prepared, directory))
    } else {
        let prepared = group::prepare(
            std::path::Path::new(&args[0]),
            std::path::Path::new(&args[2]),
        )?;
        runtime.block_on(server::serve(prepared, directory))
    }
}
