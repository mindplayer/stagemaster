use crate::client::{Probe, Result, elapsed, state};
use serde_json::json;
use stagemaster_package::{Archive, Program};
use std::{fmt::Write, time::Duration};
use tokio::time::sleep;

pub async fn run(probe: &Probe, archive: &Archive, last: &Program) -> Result<()> {
    let locator = probe.discover(None).await?;
    probe.connect(&locator).await?;
    let initial = probe.read().await?;
    let digest = hex(archive.digest());
    if !state(&initial)["owner"].is_null() || state(&initial)["instance"].is_string() {
        return Err("当前已有控制者或运行实例，测试不接管".into());
    }
    assert_eq!(state(&initial)["package"], digest);
    assert_eq!(initial["reply"]["programCount"], archive.entries().len());
    let boot = initial["reply"]["boot"].clone();
    let mut keys = Vec::new();
    for (index, entry) in archive.entries().iter().enumerate() {
        let page = probe.json(json!({"kind":"catalog","epoch":probe.epoch()?,"revision":initial["reply"]["revision"],"index":index})).await?;
        let program = &page["reply"]["body"]["program"];
        let id = hex(&entry.id);
        assert_eq!(program["key"]["id"], id);
        assert_eq!(program["name"], entry.name);
        keys.push(program["key"].clone());
    }
    // Four distinct near-budget shapes receive full state-machine exercise.
    // A full catalogue loads each of its 64 entries once through the same runtime.
    for (index, key) in keys.iter().enumerate() {
        probe
            .apply(json!({"kind":"acquire","takeover":false}))
            .await?;
        probe.apply(json!({"kind":"select","program":key})).await?;
        let loaded = probe.apply(json!({"kind":"load"})).await?;
        assert_eq!(state(&loaded)["loaded"], *key);
        assert_eq!(loaded["reply"]["boot"], boot);
        let page = probe.json(json!({"kind":"step","epoch":probe.epoch()?,"revision":loaded["reply"]["revision"],"index":0})).await?;
        let step = page["reply"]["body"]["step"]["id"]
            .as_str()
            .ok_or("没有步骤")?;
        let running = probe.apply(json!({"kind":"start","step":step})).await?;
        assert_eq!(state(&running)["status"], "running");
        if keys.len() == 4 {
            state_machine(probe, loaded["reply"]["stepCount"].as_u64().unwrap()).await?;
        } else {
            sleep(Duration::from_millis(300)).await;
        }
        if index + 1 == keys.len() {
            offline(probe, &locator, &boot, last).await?;
        }
        let stopped = probe.apply(json!({"kind":"stop"})).await?;
        assert_eq!(stopped["reply"]["boot"], boot);
        assert_eq!(state(&stopped)["status"], "idle");
        assert!(state(&stopped)["instance"].is_null());
        probe.apply(json!({"kind":"release"})).await?;
        println!("CAPACITY PROGRAM index={index} passed=true");
    }
    let final_state = probe.read().await?;
    assert_eq!(state(&final_state)["status"], "idle");
    assert!(state(&final_state)["owner"].is_null());
    probe.disconnect().await?;
    println!(
        "PASS：容量验收 {} 节目；同启动、逐个载入与运行、停止归还；物理发送禁用",
        keys.len()
    );
    Ok(())
}
async fn state_machine(probe: &Probe, steps: u64) -> Result<()> {
    sleep(Duration::from_secs(5)).await;
    let paused = probe.apply(json!({"kind":"pause"})).await?;
    assert_eq!(state(&paused)["status"], "paused");
    sleep(Duration::from_secs(2)).await;
    assert_eq!(elapsed(&probe.read().await?), elapsed(&paused));
    probe.apply(json!({"kind":"resume"})).await?;
    if steps > 1 {
        probe.apply(json!({"kind":"next"})).await?;
    }
    sleep(Duration::from_secs(3)).await;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, b| {
        write!(text, "{b:02x}").unwrap();
        text
    })
}

async fn offline(
    probe: &Probe,
    locator: &str,
    boot: &serde_json::Value,
    program: &Program,
) -> Result<()> {
    let paused = probe.apply(json!({"kind":"pause"})).await?;
    sleep(Duration::from_secs(10)).await;
    assert_eq!(elapsed(&probe.read().await?), elapsed(&paused));
    probe.apply(json!({"kind":"resume"})).await?;
    let before = probe.read().await?;
    probe.disconnect().await?;
    sleep(Duration::from_secs(45)).await;
    probe.connect(locator).await?;
    let after = probe.read().await?;
    assert_eq!(after["reply"]["boot"], *boot);
    assert_eq!(state(&after)["instance"], state(&before)["instance"]);
    assert_eq!(state(&after)["status"], "running");
    let position = |view: &serde_json::Value| -> Result<(usize, u64)> {
        let index = program
            .labels
            .iter()
            .position(|l| state(view)["step"] == hex(&l.id))
            .ok_or("运行步骤不在实际节目中")?;
        Ok((index, elapsed(view)))
    };
    let observed = |view: &serde_json::Value| -> Result<u64> {
        Ok(view["reply"]["observedMs"]
            .as_str()
            .ok_or("缺少设备采样时刻")?
            .parse()?)
    };
    let delta = observed(&after)?
        .checked_sub(observed(&before)?)
        .ok_or("设备时间倒退")?;
    assert!(crate::progress::advances(
        program,
        position(&before)?,
        position(&after)?,
        delta
    ));
    assert!(state(&after)["owner"].is_null());
    probe
        .apply(json!({"kind":"acquire","takeover":false}))
        .await?;
    println!("CAPACITY OFFLINE passed=true");
    Ok(())
}
