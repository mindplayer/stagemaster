//! Explicit current-board installation acceptance using the same service as the app.
//! Explicit local development credentials, unbonded encrypted GATT (ADR-051).
//! No physical output. An optional native locator is required when several boards appear.
use stagemaster_device_host::{Ble, Phase as LinkPhase, Request, Service as Link, Snapshot};
use stagemaster_device_upload::{Phase, Prepared, Service, Task};
use std::{error::Error, sync::Arc, time::Duration};
use tokio::time::{Instant, sleep};
#[path = "install_device/faults.rs"]
mod faults;

const DEVICE: &str = "534d4553503332533300288485569774";
type Result<T> = std::result::Result<T, Box<dyn Error>>;

async fn settled(link: &Link<Ble>) -> Result<Snapshot> {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        let state = link.request(Request::Status)?;
        match state.phase {
            LinkPhase::Idle | LinkPhase::Connected => return Ok(state),
            LinkPhase::Fault | LinkPhase::Blocked => {
                return Err(format!("连接失败：{:?}", state.problem).into());
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            return Err("连接状态等待超时".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}

async fn connect(link: &Link<Ble>, epoch: u32, locator: &str) -> Result<Snapshot> {
    link.request(Request::Connect {
        epoch,
        id: locator.into(),
    })?;
    let state = settled(link).await?;
    let description = state.description.as_ref().ok_or("缺少描述")?;
    assert_eq!(description.device_id, DEVICE);
    assert_eq!(description.authentication_method, 2);
    assert!(
        state
            .diagnostics
            .as_ref()
            .is_some_and(|d| d.output_disabled && d.self_test)
    );
    assert!(link.installation_peer(state.epoch)?.is_some());
    println!("连接 {}", serde_json::to_string(&state)?);
    Ok(state)
}

async fn finished(upload: &Service<Link<Ble>>) -> Result<Task> {
    let bytes = upload.snapshot()?.task.ok_or("缺少任务")?.package.bytes;
    // This is the CLI's whole-transfer wait, not a device or per-request deadline.
    // Allow the declared 2 MiB maximum at a conservative 2 KiB/s, plus setup time.
    let seconds = 60 + u64::try_from(bytes.div_ceil(2048))?;
    let deadline = Instant::now() + Duration::from_secs(seconds.max(240));
    let mut previous = 0;
    loop {
        let snapshot = upload.snapshot()?;
        let task = snapshot.task.ok_or("缺少任务")?;
        if snapshot.revision != previous {
            println!("任务 {}", serde_json::to_string(&task)?);
            previous = snapshot.revision;
        }
        if !task.running {
            return Ok(task);
        }
        if Instant::now() >= deadline {
            return Err("安装等待超时".into());
        }
        sleep(Duration::from_millis(150)).await;
    }
}

async fn exercise(
    link: &Link<Ble>,
    upload: &Service<Link<Ble>>,
    bytes: Arc<[u8]>,
    mode: &str,
    locator: Option<&str>,
) -> Result<()> {
    let scanning = link.request(Request::Scan { epoch: 0 })?;
    assert!(matches!(
        scanning.phase,
        LinkPhase::Preparing | LinkPhase::Scanning
    ));
    let found = settled(link).await?;
    let locator = match locator {
        Some(id) if found.candidates.iter().any(|c| c.id == id) => id,
        None if found.candidates.len() == 1 => &found.candidates[0].id,
        _ => return Err("请从本次发现结果明确选择连接标识".into()),
    };
    // Locator is only a discovery hint. connect checks DEVICE before any install.
    let connected = connect(link, found.epoch, locator).await?;
    if matches!(mode, "corrupt" | "lost-commit") {
        return faults::exercise(link, connected.epoch, &bytes, mode, locator).await;
    }
    let started = upload
        .start(Prepared::new(bytes.clone())?, connected.epoch, DEVICE)?
        .task
        .ok_or("缺少任务")?;
    if mode != "install" {
        let deadline = Instant::now() + Duration::from_mins(1);
        loop {
            let task = upload.snapshot()?.task.ok_or("缺少任务")?;
            if !task.running {
                return Err("预期传输尚未开始或已提前结束，不能计为取消／断线验收".into());
            }
            if task.confirmed_bytes >= 4096 {
                break;
            }
            if Instant::now() >= deadline {
                return Err("未进入传输阶段".into());
            }
            sleep(Duration::from_millis(30)).await;
        }
        if mode == "cancel" {
            upload.cancel(&started.id)?;
        } else {
            let state = link.request(Request::Status)?;
            link.request(Request::Cancel { epoch: state.epoch })?;
            let idle = settled(link).await?;
            let interrupted = finished(upload).await?;
            assert_eq!(interrupted.phase, Phase::Reconnect);
            let next = connect(link, idle.epoch, locator).await?;
            upload.resume(&started.id, next.epoch)?;
        }
    }
    let result = finished(upload).await?;
    println!(
        "任务终态连接 {}",
        serde_json::to_string(&link.request(Request::Status)?)?
    );
    if mode == "cancel" {
        assert_eq!(result.phase, Phase::Cancelled);
    } else {
        assert_eq!(result.phase, Phase::Installed);
        let receipt = result.receipt.ok_or("缺少安装回执")?;
        assert_eq!(receipt.digest, started.package.digest);
        assert_eq!(receipt.bytes, started.package.bytes);
        // A same-connection repeated task must reconcile without another generation.
        let epoch = link.request(Request::Status)?.epoch;
        upload.start(Prepared::new(bytes)?, epoch, DEVICE)?;
        let repeated = finished(upload).await?;
        assert_eq!(repeated.phase, Phase::Installed);
        assert_eq!(
            repeated.receipt.ok_or("重复安装缺少回执")?.generation,
            receipt.generation
        );
    }
    sleep(Duration::from_secs(6)).await;
    let steady = link.request(Request::Status)?;
    assert_eq!(steady.phase, LinkPhase::Connected);
    assert!(steady.heartbeat_count >= 2);
    println!("结束连接 {}", serde_json::to_string(&steady)?);
    println!("PASS: {mode}，使用应用凭据、加密GATT、任务服务和 NOR 回执，禁止输出");
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let arguments: Vec<_> = std::env::args().collect();
    if !(3..=4).contains(&arguments.len())
        || !["install", "cancel", "resume", "corrupt", "lost-commit"]
            .contains(&arguments[2].as_str())
    {
        return Err(
            "用法：install_device <项目 data 内播放包> <install|cancel|resume|corrupt|lost-commit> [本次连接标识]"
                .into(),
        );
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let path = std::path::Path::new(&arguments[1]).canonicalize()?;
    if !path.starts_with(root.join("data")) {
        return Err("测试包必须位于当前项目 data 内".into());
    }
    let bytes: Arc<[u8]> = std::fs::read(path)?.into();
    Prepared::new(bytes.clone())?;
    let configuration = std::env::var_os("STAGEMASTER_CONTROLLER_CONFIGURATION")
        .ok_or("须显式指定控制端开发凭据")?;
    let configuration = std::path::Path::new(&configuration).canonicalize()?;
    if !configuration.starts_with(root.join("data")) {
        return Err("开发凭据必须位于当前项目data内".into());
    }
    let credentials = stagemaster_device_host::read_development_configuration(&configuration)?;
    let link = Arc::new(Link::new(Ble::with_development_configuration(credentials)));
    let upload = Service::new(link.clone());
    let result = exercise(
        &link,
        &upload,
        bytes,
        &arguments[2],
        arguments.get(3).map(String::as_str),
    )
    .await;
    upload.shutdown().await?;
    link.shutdown().await?;
    result
}
