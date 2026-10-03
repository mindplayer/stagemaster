use super::client::{Probe, Result, elapsed, state};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

pub async fn run(probe: &Probe, locator: &str, program: Value) -> Result<()> {
    let initial = probe.read().await?;
    if !state(&initial)["owner"].is_null() || state(&initial)["instance"].is_string() {
        return Err("设备已有控制者或运行实例，验收不自动接管或停止".into());
    }
    probe
        .apply(json!({"kind":"acquire","takeover":false}))
        .await?;
    probe
        .apply(json!({"kind":"select","program":program}))
        .await?;
    let loaded = probe.apply(json!({"kind":"load"})).await?;
    assert_eq!(state(&loaded)["loaded"], program);
    let page = probe.json(json!({"kind":"step","epoch":probe.epoch()?,"revision":loaded["reply"]["revision"],"index":0})).await?;
    let step = page["reply"]["body"]["step"]["id"]
        .as_str()
        .ok_or("没有可执行步骤")?;
    let running = probe.apply(json!({"kind":"start","step":step})).await?;
    assert_eq!(state(&running)["status"], "running");
    let instance = state(&running)["instance"].clone();
    let boot = running["reply"]["boot"].clone();
    sleep(Duration::from_secs(3)).await;
    let paused = probe.apply(json!({"kind":"pause"})).await?;
    assert_eq!(state(&paused)["status"], "paused");
    assert!(elapsed(&paused) >= 2500);
    sleep(Duration::from_secs(2)).await;
    assert_eq!(elapsed(&probe.read().await?), elapsed(&paused));
    probe.apply(json!({"kind":"resume"})).await?;
    let before = probe.read().await?;
    probe.disconnect().await?;
    sleep(Duration::from_secs(5)).await;
    probe.connect(locator).await?;
    let after = probe.read().await?;
    assert_eq!(after["reply"]["boot"], boot);
    assert_eq!(state(&after)["instance"], instance);
    assert_eq!(state(&after)["status"], "running");
    assert!(state(&after)["owner"].is_null());
    assert!(elapsed(&after) >= elapsed(&before) + 4500);
    println!("断线后同一节目继续 {after}");
    probe
        .apply(json!({"kind":"acquire","takeover":false}))
        .await?;
    let stopped = probe.apply(json!({"kind":"stop"})).await?;
    assert_eq!(state(&stopped)["status"], "idle");
    probe.apply(json!({"kind":"release"})).await?;
    sleep(Duration::from_secs(6)).await;
    let connection = probe.snapshot()?;
    assert!(connection.heartbeat_count >= 2);
    assert!(connection.diagnostics.as_ref().unwrap().output_disabled);
    let final_state = probe.read().await?;
    assert_eq!(state(&final_state)["status"], "idle");
    assert!(state(&final_state)["owner"].is_null());
    println!("最终停止且无控制者 {final_state}");
    Ok(())
}
