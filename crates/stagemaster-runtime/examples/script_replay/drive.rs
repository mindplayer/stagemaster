use super::prepare::{Prepared, apply, grant, request};
use serde_json::{Value, json};
use stagemaster_runtime::{Action, Code, Lease, Status};
use std::{collections::BTreeSet, error::Error};

fn compare(prepared: &mut Prepared, now: u64) -> Result<Value, Box<dyn Error>> {
    prepared.runtime.tick(now)?;
    prepared.reference.advance(now)?;
    let state = prepared.runtime.state();
    let expected_step = prepared
        .reference
        .index()
        .map(|index| prepared.step_ids[index]);
    if state.status != Some(prepared.reference.status())
        || state.step != expected_step
        || state.elapsed_ms != prepared.reference.elapsed_ms()
    {
        return Err(format!("{now} 毫秒步骤／状态／经过时间不一致：{state:?}").into());
    }
    let mut actual = [0; 512];
    let frame = prepared
        .runtime
        .render(&mut actual)?
        .ok_or("执行未生成逻辑帧")?;
    let expected = prepared.output.render(prepared.reference.values())?;
    if expected.slots.len() != actual.len() {
        return Err("源工程输出不是完整 512 通道帧".into());
    }
    if let Some(slot) = actual.iter().zip(&expected.slots).position(|(a, b)| a != b) {
        return Err(format!("{now} 毫秒第 {} 通道与源工程输出不一致", slot + 1).into());
    }
    if frame.sampled_ms != now
        || frame.instance != state.instance
        || frame.program != state.loaded.unwrap()
        || frame.boot != state.boot
        || frame.revision != state.revision
        || frame.universe != expected.universe
    {
        return Err("逻辑帧身份不一致".into());
    }
    Ok(
        json!({"atMs":now,"step":state.step.map(|id|uuid::Uuid::from_bytes(id).to_string()),
        "elapsedMs":state.elapsed_ms,"status":format!("{:?}",state.status.unwrap()),
        "phase":format!("{:?}",prepared.reference.progress().phase),"controlled":state.owner.is_some()}),
    )
}
fn reconnect(p: &mut Prepared, old_lease: Lease, now: u64) -> Result<(), Box<dyn Error>> {
    p.lease = p.runtime.acquire(grant(), false, now)?;
    let state = p.runtime.state();
    let late = stagemaster_runtime::Request {
        lease: old_lease,
        serial: 1,
        expected_revision: state.revision,
        action: Action::Stop,
    };
    if p.runtime.submit(late, now) != Err(Code::Lease) {
        return Err("旧连接停止没有被拒绝".into());
    }
    let next = request(&p.runtime, p.lease, Action::Next);
    if p.runtime.submit(next, now)?.result != Err(Code::Step) {
        return Err("末步越界推进没有被拒绝".into());
    }
    Ok(())
}
pub fn run(p: &mut Prepared) -> Result<Value, Box<dyn Error>> {
    apply(
        &mut p.runtime,
        p.lease,
        Action::Start {
            step: p.step_ids[0],
        },
        0,
    )?;
    p.reference.execute(0, 0)?;
    let instance = p.runtime.state().instance.ok_or("没有显式运行实例")?;
    let old_lease = p.lease;
    let checkpoints = p.schedule.checkpoints();
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    let mut phases = BTreeSet::new();
    for now in 0..=p.schedule.stop {
        if now == p.schedule.next {
            let original = request(&p.runtime, p.lease, Action::Next);
            let receipt = p.runtime.submit(original, now)?;
            receipt.result?;
            p.reference.next(now)?;
            if p.runtime.submit(original, now)? != receipt {
                return Err("重复下一步未返回原回执".into());
            }
        }
        if now == p.schedule.pause - 1 {
            let mut old = request(&p.runtime, p.lease, Action::Stop);
            old.expected_revision -= 1;
            if p.runtime.submit(old, now)?.result != Err(Code::Revision) {
                return Err("旧修订停止操作没有被拒绝".into());
            }
        }
        if now == p.schedule.pause {
            apply(&mut p.runtime, p.lease, Action::Pause, now)?;
            p.reference.pause(now)?;
            let invalid = request(&p.runtime, p.lease, Action::Next);
            if p.runtime.submit(invalid, now)?.result != Err(Code::State) {
                return Err("暂停时下一步没有被拒绝".into());
            }
        }
        if now == p.schedule.resume {
            apply(&mut p.runtime, p.lease, Action::Resume, now)?;
            p.reference.resume(now)?;
        }
        if now == p.schedule.resume + 1 {
            p.runtime.release(p.lease, now)?;
        }
        if now == p.schedule.reconnect {
            reconnect(p, old_lease, now)?;
        }
        if now == p.schedule.stop {
            apply(&mut p.runtime, p.lease, Action::Stop, now)?;
            p.reference.stop(now)?;
        } else if p.runtime.state().instance != Some(instance) {
            return Err(format!("{now} 毫秒运行实例意外更换").into());
        }
        let record = compare(p, now)?;
        if let Some(index) = p.reference.index() {
            seen.insert(index);
            phases.insert((index, record["phase"].as_str().unwrap().to_owned()));
        }
        if checkpoints.contains(&now) {
            records.push(record);
        }
        if (p.schedule.resume + 1..p.schedule.reconnect).contains(&now)
            && p.runtime.state().owner.is_some()
        {
            return Err("归还控制后仍有控制者".into());
        }
    }
    if seen != BTreeSet::from([0, 1, 2])
        || p.reads.get() != 0
        || p.runtime.state().instance.is_some()
        || p.runtime.state().status != Some(Status::Idle)
    {
        return Err("完整三步／停止／运行时无存储访问验收未达标".into());
    }
    Ok(
        json!({"program":p.name,"sourceDigest":p.source_digest,"sourcePlanNotDecodedPackage":true,
        "framesCompared":p.schedule.stop+1,"stepsVisited":seen,"phases":phases,"checkpoints":records,
        "automaticTransitionAtMs":p.schedule.automatic,"storageReadsDuringExecution":p.reads.get(),
        "duplicateNextPreserved":true,"staleRevisionRejected":true,"pausedNextRejected":true,
        "continuedWithoutController":true,"reconnectedWithoutRestart":true,"oldLeaseRejected":true,
        "stopRestoredSourceDefaults":true,"softwareOnly":true,"physicalOutput":false,"authorizationQualified":false}),
    )
}
