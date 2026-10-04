#[path = "fixture_axis_speed.rs"]
mod axis_speed;
pub use axis_speed::{edit, setup as speed_setup};
use serde_json::{Value, json};
use stagemaster_project::Document;

pub fn definition() -> Value {
    let mut def = axis_speed::definition(false);
    def["name"] = json!("内置程序软件验收");
    let mut functions = vec![json!({"key":"external","name":"外部通道控制","mode":"slot",
        "dmxFrom":0,"dmxTo":59,"dmxDefault":0})];
    for (kind, first, last) in [("auto", 60, 159), ("sound", 160, 255)] {
        for n in 0..4 {
            let from = first + 25 * n;
            functions.push(json!({"key":format!("{kind}.{}",3-n),"name":format!("{} {}",if kind=="auto" {"自动"} else {"声控"},3-n),
                "mode":"slot","dmxFrom":from,"dmxTo":if n==3 {last} else {from+24},"dmxDefault":from}));
        }
    }
    def["channels"]
        .as_array_mut()
        .unwrap()
        .push(json!({"attribute":"fixture-program","coarse":10,
        "fine":null,"defaultValue":{"functionKey":"external","position":0},"functions":functions}));
    def
}
pub fn save(doc: &mut Document, definition: Value) -> Result<(), String> {
    let mut command = json!({"op":"fixture","command":{"op":"saveProfile"}});
    command["command"]["definition"] = definition;
    axis_speed::edit(doc, command)
}
pub fn setup(def: Value) -> (Document, Vec<String>, Vec<String>) {
    let mut doc = Document::new("内置程序软件验收").unwrap();
    save(&mut doc, def).unwrap();
    let view = doc.view();
    for (name, address) in [("甲灯", 17), ("乙灯", 28)] {
        axis_speed::edit(
            &mut doc,
            json!({"op":"addFixture","name":name,
            "profileId":view.profiles.last().unwrap().id,"domainId":view.domains[0].id,
            "universe":1,"address":address}),
        )
        .unwrap();
    }
    for name in ["开始", "目标"] {
        axis_speed::edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let view = doc.view();
    (
        doc,
        view.fixtures.iter().map(|f| f.id.clone()).collect(),
        view.scenes.iter().map(|s| s.id.clone()).collect(),
    )
}
pub fn choose(
    doc: &mut Document,
    scene: &str,
    fixture: &str,
    key: &str,
    position: u16,
) -> Result<(), String> {
    axis_speed::edit(
        doc,
        json!({"op":"setSceneFunctionValue","sceneId":scene,
        "fixtureId":fixture,"attribute":"fixture-program",
        "selection":{"functionKey":key,"position":position}}),
    )
}
pub fn raw(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
pub fn decode(root: &Value) -> Result<Document, String> {
    Document::decode(&serde_json::to_vec(root).unwrap())
}
