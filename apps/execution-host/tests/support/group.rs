#[allow(dead_code)]
#[path = "../../../../crates/stagemaster-project/tests/support/fixture_function.rs"]
mod fixtures;
use super::{Harness, ok};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub fn id(n: u8) -> String {
    format!("66666666-0000-4000-8000-{n:012}")
}
pub fn write(path: &Path) -> PathBuf {
    let (mut doc, fixture, scenes) = fixtures::setup(false);
    fixtures::choose(&mut doc, &scenes[0], &fixture, "color-wheel", "red", 0).unwrap();
    fixtures::choose(&mut doc, &scenes[1], &fixture, "color-wheel", "blue", 0).unwrap();
    fixtures::edit(&mut doc,json!({"op":"effect","command":{"kind":"put","sceneId":scenes[0],"effect":{
        "id":id(90),"name":"呼吸","enabled":true,"fixtureIds":[fixture],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":10_000,"high":50_000}]
    }}})).unwrap();
    fixtures::edit(
        &mut doc,
        json!({"op":"sequence","command":{"kind":"add","name":"蓝色保持","sceneId":scenes[1]}}),
    )
    .unwrap();
    let mut raw = fixtures::raw(&doc);
    raw["lighting"]["sequences"][0]["steps"][0]["fade"] =
        json!({"ticks":"0","ticksPerSecond":"1000"});
    let doc = fixtures::decode(&raw).unwrap();
    fs::write(path, doc.encode().unwrap()).unwrap();
    let manifest = path.with_file_name("sources.json");
    fs::write(&manifest,serde_json::to_vec(&json!({"version":1,"sources":[
        {"id":id(1),"priority":0,"selection":{"kind":"scene","id":scenes[0]}},
        {"id":id(2),"priority":0,"selection":{"kind":"sequence","id":doc.view().sequences[0].id}},
        {"id":id(3),"priority":0,"selection":{"kind":"manual"}}
    ]})).unwrap()).unwrap();
    manifest
}
pub fn action(source: u8, action: Value) -> Value {
    {
        let mut value = json!({"kind":"source","source":id(source)});
        value["action"] = action;
        value
    }
}
pub async fn apply(
    h: &Harness,
    session: &str,
    serial: u64,
    revision: &Value,
    source: u8,
    command: Value,
) -> Value {
    h.command(
        session,
        serial,
        json!({"kind":"submit","expectedRevision":revision,"action":action(source,command)}),
    )
    .await
}
pub async fn start(
    h: &Harness,
    session: &str,
    serial: u64,
    revision: &Value,
    index: usize,
) -> Value {
    let source = ok(h.get("/source")).await;
    let id = u8::try_from(index + 1).unwrap();
    apply(
        h,
        session,
        serial,
        revision,
        id,
        json!({"kind":"start","step":source["sources"][index]["steps"][0]["id"]}),
    )
    .await
}
pub async fn until(h: &Harness, predicate: impl Fn(&Value) -> bool) -> Value {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = h.snapshot().await;
        if predicate(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < end, "多来源后台未达到预期状态");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
