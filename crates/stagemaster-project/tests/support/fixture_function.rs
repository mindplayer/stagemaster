use serde_json::{Value, json};
use stagemaster_project::{Document, EditCommand};
pub fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value::<EditCommand>(command).unwrap())
}
pub fn definition(fine: bool) -> Value {
    let slot = |key: &str, name: &str, from: u16, to: u16, representative: u16| {
        json!({
        "key":key,"name":name,"mode":"slot","dmxFrom":from,"dmxTo":to,"dmxDefault":representative})
    };
    let channel = |attribute: &str, coarse: u16, default: &str, functions: Vec<Value>| {
        json!({
        "attribute":attribute,"coarse":coarse,"fine":null,
        "defaultValue":{"functionKey":default,"position":0},"functions":functions})
    };
    let mut value = json!({"name":"功能测试灯","manufacturer":"契约夹具","model":"区间测试","mode":"功能通道",
        "footprint":if fine {6} else {5},"channels":[
            {"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":0},
            channel("color-wheel",2,"white",vec![slot("white","白色",0,15,0),slot("red","红色",16,31,20),slot("blue","蓝色",32,47,40)]),
            channel("gobo-wheel",3,"open",vec![slot("open","通光",0,15,0),slot("dots","圆点",16,31,25)]),
            channel("shutter",4,"open",vec![slot("closed","关闭",0,15,0),slot("open","常开",16,31,20),
                json!({"key":"strobe","name":"频闪","mode":"range","dmxFrom":32,"dmxTo":239,"dmxDefault":64})]),
            channel("prism",5,"off",vec![slot("off","退出",0,127,0),slot("on","插入",128,255,200)])]});
    if fine {
        value["channels"][1]["fine"] = json!(6);
    }
    value
}
pub fn setup(fine: bool) -> (Document, String, Vec<String>) {
    let mut doc = Document::new("功能区间验收").unwrap();
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition(fine)}}),
    )
    .unwrap();
    let v = doc.view();
    let p = v.profiles.last().unwrap();
    edit(&mut doc,json!({"op":"addFixture","name":"功能灯","profileId":p.id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap();
    for name in ["入场", "变换"] {
        edit(&mut doc, json!({"op":"addScene","name":name})).unwrap();
    }
    let v = doc.view();
    (
        doc,
        v.fixtures[0].id.clone(),
        v.scenes.into_iter().map(|s| s.id).collect(),
    )
}
pub fn choose(
    doc: &mut Document,
    scene: &str,
    fixture: &str,
    attribute: &str,
    key: &str,
    position: u16,
) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"setSceneFunctionValue","sceneId":scene,"fixtureId":fixture,"attribute":attribute,
        "selection":{"functionKey":key,"position":position}}),
    )
}
pub fn raw(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
pub fn decode(value: &Value) -> Result<Document, String> {
    Document::decode(&serde_json::to_vec(value).unwrap())
}
