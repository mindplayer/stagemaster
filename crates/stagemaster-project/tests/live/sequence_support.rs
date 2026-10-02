use super::support::{edit, setup};
use serde_json::{Value, json};
use stagemaster_engine::live::{Frame, Handle, LiveMixer};
use stagemaster_project::Document;

pub fn fixture(
    tracking: &str,
    repeat: bool,
    timings: &[(u64, u64, Option<u64>)],
) -> (Document, String) {
    let (mut doc, fixture, _) = setup();
    edit(&mut doc, json!({"op":"addScene","name":"释放位置"}));
    let mut raw: Value = serde_json::from_slice(&doc.encode().unwrap()).unwrap();
    let scenes = raw["lighting"]["scenes"].as_array_mut().unwrap();
    for scene in scenes.iter_mut() {
        scene["effects"] = json!([]);
    }
    scenes[0]["assignments"] = json!([set(&fixture, "pan", 50_000), set(&fixture, "red", 20_000)]);
    scenes[1]["assignments"] = json!([set(&fixture, "dimmer", 40_000)]);
    scenes[2]["assignments"] = json!([{"target":{"fixtureId":fixture,"attribute":"pan"},"operation":"release"},set(&fixture,"dimmer",10_000)]);
    let steps:Vec<Value>=timings.iter().enumerate().map(|(i,(delay,fade,wait))| json!({
        "id":format!("99999999-0000-4000-8000-00000000001{i}"),"name":format!("步骤 {i}"),"number":(i+1).to_string(),"sceneId":scenes[i]["id"],"actionIds":[],
        "delay":time(*delay),"fade":time(*fade),"advance":wait.map_or_else(||json!({"kind":"manual"}),|wait|json!({"kind":"after","wait":time(wait)}))
    })).collect();
    let id = "99999999-0000-4000-8000-000000000020".to_owned();
    raw["lighting"]["sequences"] = json!([{"id":id,"name":"多来源列表","release":"profile-defaults","tracking":tracking,"repeat":if repeat {"loop"} else {"once"},"steps":steps}]);
    (decode(&raw), id)
}
fn time(ms: u64) -> Value {
    json!({"ticks":ms.to_string(),"ticksPerSecond":"1000"})
}
pub fn set(fixture: &str, key: &str, value: u16) -> Value {
    json!({"target":{"fixtureId":fixture,"attribute":key},"operation":"set","source":{"kind":"literal","value":{"kind":"normalized","value":value}}})
}
pub fn decode(raw: &Value) -> Document {
    Document::decode(&serde_json::to_vec(raw).unwrap()).unwrap()
}
pub fn raw(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
pub fn hand(m: &mut LiveMixer, h: Handle, serial: u64, pan: u16) {
    m.publish(
        h,
        Frame {
            layout: m.layout().id(),
            serial,
            values: &[None, Some(pan), None, None, None, None, None],
            assert: &[false, true, false, false, false, false, false],
        },
    )
    .unwrap();
}
pub fn winners(m: &LiveMixer) -> [Option<Handle>; 7] {
    let mut winners = [None; 7];
    m.render(&mut [0; 7], &mut winners).unwrap();
    winners
}
