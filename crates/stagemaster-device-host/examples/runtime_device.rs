//! Explicit real-board validation of the same JSON application port as the desktop.
//! No protocol simulator, credentials in logs, package writes or physical output.
#[path = "runtime_device/client.rs"]
mod client;
#[path = "runtime_device/exercise.rs"]
mod exercise;
use client::{Probe, Result};
use serde_json::{Value, json};
use stagemaster_device_host::{Ble, Service, runtime_ui::ExpectedAccess};
use std::path::Path;

async fn run(probe: &Probe, mode: &str, locator: Option<&str>) -> Result<()> {
    let locator = probe.discover(locator).await?;
    probe.connect(&locator).await?;
    let initial = probe.read().await?;
    println!("初始运行状态 {initial}");
    let count = initial["reply"]["programCount"]
        .as_u64()
        .ok_or("缺少目录计数")?;
    let revision = &initial["reply"]["revision"];
    let mut scene: Option<Value> = None;
    for index in 0..count {
        let page = probe
            .json(
                json!({"kind":"catalog","epoch":probe.epoch()?,"revision":revision,"index":index}),
            )
            .await?;
        let program = &page["reply"]["body"]["program"];
        if program.is_null() {
            return Err("设备目录不完整".into());
        }
        println!("节目 {program}");
        if scene.is_none() && program["key"]["kind"] == "scene" {
            scene = Some(program["key"].clone());
        }
    }
    if mode == "exercise" {
        exercise::run(
            probe,
            &locator,
            scene.ok_or("当前目录没有可用于验收的保持场景")?,
        )
        .await?;
    }
    println!("结束连接 {}", serde_json::to_string(&probe.snapshot()?)?);
    println!("PASS：{mode}，真实 GATT／原生 JSON／设备 Runtime；物理发送禁用");
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) || !["observe", "exercise"].contains(&args[1].as_str()) {
        return Err(
            "用法：runtime_device <项目 data 内控制端配置> <observe|exercise> [本次发现标识]"
                .into(),
        );
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let path = Path::new(&args[0]).canonicalize()?;
    if !path.starts_with(root.join("data")) {
        return Err("配置必须位于当前项目 data 内".into());
    }
    let config = stagemaster_device_host::read_development_configuration(&path)?;
    let access = ExpectedAccess::from_configuration(&config);
    let probe = Probe {
        link: Service::new(Ble::with_development_configuration(config)),
        access,
    };
    let result = run(&probe, &args[1], args.get(2).map(String::as_str)).await;
    probe.link.shutdown().await?;
    result
}
