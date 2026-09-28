//! Explicit current-board rejection acceptance. No installation or physical output.
use stagemaster_device_auth::application::{Configuration, Role};
use stagemaster_device_host::{Ble, Phase, Request, Service, Snapshot};
use stagemaster_device_session::SecretKey;
use std::{io::Read, path::Path, time::Duration};
use tokio::time::{Instant, sleep};
use zeroize::Zeroizing;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const DEVICE: &str = "534d4553503332533300288485569774";

async fn settled(service: &Service<Ble>) -> Result<Snapshot> {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        let state = service.request(Request::Status)?;
        if matches!(
            state.phase,
            Phase::Idle | Phase::Connected | Phase::Fault | Phase::Blocked
        ) {
            return Ok(state);
        }
        if Instant::now() >= deadline {
            return Err("等待状态超时".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}
fn backend(original: &[u8], mode: &str) -> Result<Ble> {
    if mode == "none" {
        return Ok(Ble::default());
    }
    let mut bytes = Zeroizing::new([0; 160]);
    bytes.copy_from_slice(original);
    let mut random = Zeroizing::new([0; 32]);
    getrandom::fill(random.as_mut()).map_err(|_| "随机源失败")?;
    let public = SecretKey::import(*random)?.public();
    match mode {
        "unknown-controller" => {
            bytes[56..88].copy_from_slice(random.as_ref());
            bytes[120..152].copy_from_slice(&public);
        }
        "wrong-device-key" => bytes[88..120].copy_from_slice(&public),
        "wrong-principal" => bytes[24] ^= 1,
        _ => return Err("未知测试模式".into()),
    }
    Ok(Ble::with_development_configuration(Configuration::import(
        bytes.as_ref(),
        Role::Controller,
    )?))
}
async fn exercise(service: &Service<Ble>, mode: &str) -> Result<()> {
    service.request(Request::Scan { epoch: 0 })?;
    let found = settled(service).await?;
    if found.phase != Phase::Idle || found.candidates.len() != 1 {
        return Err("须恰好发现一台当前受控设备".into());
    }
    service.request(Request::Connect {
        epoch: found.epoch,
        id: found.candidates[0].id.clone(),
    })?;
    let connected = settled(service).await?;
    if mode == "none" {
        assert_eq!(connected.phase, Phase::Connected);
        let description = connected.description.as_ref().ok_or("缺少描述")?;
        assert_eq!(description.device_id, DEVICE);
        assert_eq!(description.authentication_method, 2);
        assert!(description.declared_functions.contains(&"节目安装"));
        assert!(service.installation_peer(connected.epoch)?.is_none());
        sleep(Duration::from_secs(6)).await;
        let steady = service.request(Request::Status)?;
        assert_eq!(steady.phase, Phase::Connected);
        assert!(steady.heartbeat_count >= 2);
        assert!(service.installation_peer(steady.epoch)?.is_none());
        assert!(
            steady
                .diagnostics
                .is_some_and(|d| d.output_disabled && d.self_test)
        );
    } else {
        assert_eq!(connected.phase, Phase::Fault, "错误凭据必须拒绝安全连接");
        assert!(service.installation_peer(connected.epoch).is_err());
        assert!(connected.description.is_none());
    }
    println!("PASS: {mode}，没有安装权限，没有发送存储请求");
    Ok(())
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let argument = std::env::args()
        .nth(1)
        .ok_or("须指定项目data内控制端配置")?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .canonicalize()?;
    let path = Path::new(&argument).canonicalize()?;
    if !path.starts_with(root) {
        return Err("凭据必须位于项目data内".into());
    }
    stagemaster_device_host::read_development_configuration(&path)?;
    let mut original = Zeroizing::new([0; 160]);
    std::fs::File::open(path)?.read_exact(original.as_mut())?;
    for mode in [
        "none",
        "wrong-device-key",
        "unknown-controller",
        "wrong-principal",
    ] {
        let service = Service::new(backend(original.as_ref(), mode)?);
        let result = exercise(&service, mode).await;
        service.shutdown().await?;
        result?;
    }
    Ok(())
}
