//! Explicit read-only real-device acceptance; never installs or outputs light.
use stagemaster_device_host::{Ble, Phase, Request, Service, Snapshot};
use std::{error::Error, time::Duration};
use tokio::time::{Instant, sleep};

async fn settled(service: &Service<Ble>) -> Result<Snapshot, Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        let snapshot = service.request(Request::Status)?;
        if matches!(snapshot.phase, Phase::Fault | Phase::Blocked) {
            return Err(format!("设备检查失败：{:?}", snapshot.problem).into());
        }
        if matches!(snapshot.phase, Phase::Idle | Phase::Connected) {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err("等待设备状态超时".into());
        }
        sleep(Duration::from_millis(100)).await;
    }
}

async fn inspect(service: &Service<Ble>) -> Result<(), Box<dyn Error>> {
    service.request(Request::Scan { epoch: 0 })?;
    let found = settled(service).await?;
    if found.candidates.len() != 1 {
        return Err(format!("需要恰好一台诊断设备，实际 {} 台", found.candidates.len()).into());
    }
    service.request(Request::Connect {
        epoch: found.epoch,
        id: found.candidates[0].id.clone(),
    })?;
    let connected = settled(service).await?;
    let description = connected.description.as_ref().ok_or("缺少设备描述")?;
    assert_eq!(description.firmware, "0.2.0");
    assert_eq!(description.authentication_method, 0);
    assert_eq!(description.declared_functions, ["连接诊断"]);
    assert!(
        connected
            .diagnostics
            .as_ref()
            .is_some_and(|d| d.self_test && d.output_disabled)
    );
    let device = description.device_id.clone();
    let boot = description.boot_id.clone();
    sleep(Duration::from_secs(12)).await;
    let steady = service.request(Request::Status)?;
    assert_eq!(steady.phase, Phase::Connected);
    assert!(steady.heartbeat_count >= 5);
    let description = steady.description.as_ref().ok_or("保活后缺少描述")?;
    assert_eq!(description.device_id, device);
    assert_eq!(description.boot_id, boot);
    println!("{}", serde_json::to_string(&steady)?);
    service.request(Request::Cancel {
        epoch: steady.epoch,
    })?;
    let disconnected = settled(service).await?;
    assert_eq!(disconnected.phase, Phase::Idle);
    assert!(disconnected.description.is_none() && disconnected.diagnostics.is_none());
    println!("PASS: 原生宿主读取、保活、身份保持、断开清除；禁止输出");
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let service = Service::new(Ble::default());
    let result = inspect(&service).await;
    service.shutdown().await?;
    result
}
