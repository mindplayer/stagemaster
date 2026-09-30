use serde_json::{Value, json};
use stagemaster_project::Document;

pub fn edit(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(serde_json::from_value(command).unwrap())
}
pub fn setup(fine: bool) -> (Document, String, Vec<String>) {
    let mut doc = Document::new("运动测试").unwrap();
    let definition = json!({"name":"两轴灯","manufacturer":"测试","model":"正交","mode":"运动","footprint":5,
        "channels":[{"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":12345},
        {"attribute":"pan","coarse":4,"fine":if fine { Some(2) } else { None },"defaultValue":32768},
        {"attribute":"tilt","coarse":5,"fine":if fine { Some(3) } else { None },"defaultValue":32768}],
        "positioning":{"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":true},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}}});
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition}}),
    )
    .unwrap();
    let v = doc.view();
    for address in [1, 6] {
        edit(&mut doc,json!({"op":"addFixture","name":format!("灯 {address}"),"profileId":v.profiles.last().unwrap().id,"domainId":v.domains[0].id,"universe":1,"address":address})).unwrap();
    }
    edit(&mut doc, json!({"op":"addScene","name":"运动场景"})).unwrap();
    let v = doc.view();
    (
        doc,
        v.scenes[0].id.clone(),
        v.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
pub fn motion(ids: &[String]) -> Value {
    json!({"id":"39999999-0000-4000-8000-000000000001","name":"圆形","enabled":true,"fixtureIds":ids,
        "periodMs":4096,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"position","dutyPercent":25,
        "channels":[{"attribute":"pan","amplitudeDegrees":"30","offsetDegrees":"5","phaseDegrees":0},
        {"attribute":"tilt","amplitudeDegrees":"15","offsetDegrees":"-3","phaseDegrees":90}]})
}
pub fn put(doc: &mut Document, scene: &str, effect: &Value) -> Result<(), String> {
    edit(
        doc,
        json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":effect}}),
    )
}
pub fn set(doc: &mut Document, scene: &str, fixture: &str, key: &str, value: u16) {
    edit(doc,json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"mode":"literal","value":value})).unwrap();
}
pub fn raw(doc: &Document) -> Value {
    serde_json::from_slice(&doc.encode().unwrap()).unwrap()
}
pub fn decode(root: &Value) -> Document {
    Document::decode(&serde_json::to_vec(root).unwrap()).unwrap()
}
