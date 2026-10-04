use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
pub fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(command).map_err(|e| e.to_string())?)
}
pub fn definition() -> Value {
    // Known channel locations, test defaults only; not a complete or measured 18CH profile.
    json!({"name":"独立光源软件子集","manufacturer":"测试","model":"18CH 位置子集",
      "mode":"软件验收，非实灯档案","footprint":18,"emitters":[
        {"key":"pattern","name":"图案光源"},{"key":"wash","name":"染色光源"}],"channels":[
        {"attribute":"dimmer","coarse":6,"fine":null,"defaultValue":65535},
        {"attribute":"emitter.pattern.dimmer","coarse":7,"fine":null,"defaultValue":32768},
        {"attribute":"emitter.wash.red","coarse":11,"fine":null,"defaultValue":65535},
        {"attribute":"emitter.wash.green","coarse":12,"fine":null,"defaultValue":0},
        {"attribute":"emitter.wash.blue","coarse":13,"fine":null,"defaultValue":0},
        {"attribute":"emitter.wash.white","coarse":14,"fine":null,"defaultValue":0}]})
}
pub fn save(doc: &mut Document, definition: Value) -> Result<(), String> {
    let mut command = json!({"op":"fixture","command":{"op":"saveProfile","definition":null}});
    command["command"]["definition"] = definition;
    edit(doc, command)
}
pub fn setup(def: Value) -> (Document, Vec<String>, String) {
    let mut doc = Document::new("独立光源软件验收").unwrap();
    save(&mut doc, def).unwrap();
    let view = doc.view();
    for (name, address) in [("甲灯", 17), ("乙灯", 35)] {
        edit(
            &mut doc,
            json!({"op":"addFixture","name":name,"profileId":view.profiles.last().unwrap().id,
          "domainId":view.domains[0].id,"universe":1,"address":address}),
        )
        .unwrap();
    }
    edit(&mut doc, json!({"op":"addScene","name":"独立调光"})).unwrap();
    let view = doc.view();
    (
        doc,
        view.fixtures.iter().map(|f| f.id.clone()).collect(),
        view.scenes[0].id.clone(),
    )
}
