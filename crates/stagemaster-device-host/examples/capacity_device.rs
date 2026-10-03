//! Explicit current-board capacity acceptance. Only accepts the named test corpus.
#[path = "runtime_device/client.rs"]
mod client;
#[path = "capacity_device/flow.rs"]
mod flow;
#[path = "capacity_device/progress.rs"]
mod progress;
use client::{Probe, Result};
use stagemaster_device_host::{Ble, Service, runtime_ui::ExpectedAccess};
use stagemaster_package::Archive;
use std::path::Path;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("用法：capacity_device 项目data内控制端配置 项目data内压力包".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let configuration = Path::new(&args[0]).canonicalize()?;
    let package = Path::new(&args[1]).canonicalize()?;
    if !configuration.starts_with(root.join("data"))
        || !["data/MEMORY-002", "data/MEMORY-003"]
            .iter()
            .any(|directory| package.starts_with(root.join(directory)))
    {
        return Err("配置与测试包必须位于规定的项目 data 目录".into());
    }
    let bytes = std::fs::read(package)?;
    let archive = Archive::open(bytes.as_slice())?;
    if archive.source().project_id != [0x72; 16]
        || archive.source().project_name != "ESP32 容量验收"
        || ![4, 64].contains(&archive.entries().len())
    {
        return Err("不是本轮容量测试语料".into());
    }
    let config = stagemaster_device_host::read_development_configuration(&configuration)?;
    let access = ExpectedAccess::from_configuration(&config);
    let probe = Probe {
        link: Service::new(Ble::with_development_configuration(config)),
        access,
    };
    let last = archive.load(bytes.as_slice(), archive.entries().len() - 1)?;
    let result = flow::run(&probe, &archive, &last).await;
    probe.link.shutdown().await?;
    result
}
