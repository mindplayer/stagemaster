use super::client::{Probe, Result, state};
use serde_json::json;
use std::time::Duration;
use tokio::time::{Instant, sleep};

pub async fn run(probe: &Probe, enter: bool) -> Result<()> {
    let current = probe.read().await?;
    if !state(&current)["owner"].is_null() || state(&current)["instance"].is_string() {
        return Err("设备已有控制者或节目，验收不自动接管或停止".into());
    }
    let desired = if enter { "maintenance" } else { "operation" };
    if state(&current)["mode"] == desired {
        return Ok(());
    }
    probe
        .apply(json!({"kind":"acquire","takeover":false}))
        .await?;
    probe
        .apply(json!({"kind":if enter { "beginMaintenance" } else { "finishMaintenance" }}))
        .await?;
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if state(&probe.read().await?)["mode"] == desired {
            break;
        }
        if Instant::now() >= until {
            return Err("维护切换未得到真实静默确认".into());
        }
        sleep(Duration::from_millis(50)).await;
    }
    probe.apply(json!({"kind":"release"})).await?;
    println!("PASS：维护模式 {desired}，无控制者；物理发送禁用");
    Ok(())
}
