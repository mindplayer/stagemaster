use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};

pub fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(command).map_err(|e| e.to_string())?)
}

// Known 11CH channel locations only. Defaults are test data, not physical measurements.
pub fn definition(fine: bool) -> Value {
    json!({"name":"两轴速度软件验收","manufacturer":"资料未署名","model":"30W 通道位置子集",
        "mode":"11CH 软件子集，非实灯档案","positioning":null,"footprint":11,"channels":[
        {"attribute":"pan","coarse":1,"fine":2,"defaultValue":0x1234},
        {"attribute":"tilt","coarse":3,"fine":4,"defaultValue":0xabcd},
        {"attribute":"dimmer","coarse":8,"fine":null,"defaultValue":65535},
        {"attribute":"pan-tilt-speed","coarse":9,"fine":if fine {Some(6)} else {None},"defaultValue":0x3456}]})
}

pub fn setup(fine: bool) -> (Document, Vec<String>, Vec<String>) {
    let mut doc = Document::new("两轴速度控制测试").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition(fine)}}),
    )
    .unwrap();
    let view = doc.view();
    for (name, address) in [("甲灯", 17), ("乙灯", 28)] {
        edit(
            &mut doc,
            json!({"op":"addFixture","name":name,"profileId":view.profiles.last().unwrap().id,
            "domainId":view.domains[0].id,"universe":1,"address":address}),
        )
        .unwrap();
    }
    for name in ["开始", "目标"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let view = doc.view();
    (
        doc,
        view.fixtures.iter().map(|f| f.id.clone()).collect(),
        view.scenes.iter().map(|s| s.id.clone()).collect(),
    )
}
