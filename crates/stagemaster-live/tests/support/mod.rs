#![allow(dead_code)]
use serde_json::{Value, json};
use stagemaster_live::SourceSpec;
use stagemaster_project::{Document, PackageSelection};

pub fn id(n: u64) -> String {
    format!("99999999-0000-4000-8000-{n:012}")
}
pub fn raw() -> Value {
    let mut r: Value = serde_json::from_slice(include_bytes!(
        "../../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    r["entryPoints"] = json!([]);
    r
}
pub fn decode(r: &Value) -> Document {
    Document::decode(&serde_json::to_vec(r).unwrap()).unwrap()
}
pub fn set(key: &str, value: u16) -> Value {
    json!({"target":{"fixtureId":"00000000-0000-4000-8000-000000000020","attribute":key},"operation":"set","source":{"kind":"literal","value":{"kind":"normalized","value":value}}})
}
pub fn step(n: u64, scene: u64, delay: u64, wait: Option<u64>) -> Value {
    let time = |ms: u64| json!({"ticks":ms.to_string(),"ticksPerSecond":"1000"});
    json!({"id":id(n),"name":"步骤","number":n.to_string(),"sceneId":id(scene),"actionIds":[],
        "delay":time(delay),"fade":time(0),"advance":wait.map_or_else(||json!({"kind":"manual"}),|w|json!({"kind":"after","wait":time(w)}))})
}
pub fn list(n: u64, steps: &[Value], repeat: bool) -> Value {
    json!({"id":id(n),"name":"列表","tracking":"inherited","repeat":if repeat {"loop"} else {"once"},"release":"profile-defaults","steps":steps})
}
pub fn fixture(second_delay: u64) -> Value {
    let mut r = raw();
    r["lighting"]["scenes"] = json!([
        {"id":id(1),"name":"红色起点","assignments":[set("red",10_000)]},
        {"id":id(2),"name":"只改亮度","assignments":[set("dimmer",25_000)]},
        {"id":id(3),"name":"红色终点","assignments":[set("red",30_000)]},
        {"id":id(4),"name":"竞争红色","assignments":[set("red",20_000)]},
        {"id":id(5),"name":"保持亮度","assignments":[set("dimmer",50_000)]}
    ]);
    r["lighting"]["sequences"] = json!([
        list(
            20,
            &[
                step(10, 1, 0, Some(60)),
                step(11, 2, 0, Some(20)),
                step(12, 3, 0, None)
            ],
            false
        ),
        list(21, &[step(13, 4, second_delay, None)], false)
    ]);
    r
}
pub fn specs() -> Vec<SourceSpec> {
    vec![
        playback(1, PackageSelection::Sequence { id: id(20) }),
        playback(2, PackageSelection::Sequence { id: id(21) }),
        SourceSpec {
            id: [3; 16],
            priority: 0,
            playback: None,
        },
    ]
}
pub fn playback(n: u8, selection: PackageSelection) -> SourceSpec {
    SourceSpec {
        id: [n; 16],
        priority: 0,
        playback: Some(selection.into()),
    }
}
