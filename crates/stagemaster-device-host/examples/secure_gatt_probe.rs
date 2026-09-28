//! Explicit, read-only real GATT acceptance. Trust input comes from this board's USB log.
#[path = "secure_probe/discovery.rs"]
mod discovery;
#[path = "secure_probe/flow.rs"]
mod flow;
#[path = "secure_probe/link.rs"]
mod link;
#[path = "secure_probe/notifications.rs"]
mod notifications;
use serde::Deserialize;
use std::{error::Error, path::PathBuf, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Trust {
    device: [u8; 16],
    boot: [u8; 16],
    public_key: [u8; 32],
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().ok_or("须提供当前 USB 取得的公钥文件")?);
    let mode = args.next().unwrap_or_else(|| "normal".into());
    if args.next().is_some() {
        return Err("参数过多".into());
    }
    if !matches!(
        mode.as_str(),
        "normal"
            | "small"
            | "fast"
            | "small-fast"
            | "stress"
            | "bad-key"
            | "bad-context"
            | "tamper"
            | "replay"
            | "gap"
            | "duplicate"
            | "partial"
            | "plain-only"
            | "cancel"
            | "all"
            | "fast-all"
    ) {
        return Err("未知实验模式".into());
    }
    let trust: Trust = serde_json::from_slice(&std::fs::read(path)?)?;
    if trust.device == [0; 16] || trust.boot == [0; 16] || trust.public_key == [0; 32] {
        return Err("USB 信任数据缺少身份或公钥".into());
    }
    let adapter = discovery::adapter().await?;
    let modes: Vec<&str> = if mode == "fast-all" {
        vec![
            "fast",
            "small-fast",
            "bad-context",
            "tamper",
            "replay",
            "gap",
            "duplicate",
            "partial",
            "plain-only",
            "cancel",
            "fast",
        ]
    } else if mode == "all" {
        vec![
            "normal",
            "small",
            "bad-key",
            "tamper",
            "replay",
            "gap",
            "duplicate",
            "partial",
            "plain-only",
            "cancel",
            "normal",
        ]
    } else {
        vec![mode.as_str()]
    };
    let fast_all = mode == "fast-all";
    for mode in modes {
        let mut link = link::Link::connect(
            &adapter,
            &trust,
            if mode.starts_with("small") { 20 } else { 244 },
            fast_all || mode.contains("fast") || mode == "stress",
        )
        .await?;
        let result =
            tokio::time::timeout(Duration::from_secs(65), flow::run(&mut link, &trust, mode)).await;
        let cleanup = link.close().await;
        result??;
        cleanup?;
        println!("PASS mode={mode}; no install permission; no DMX");
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    Ok(())
}
