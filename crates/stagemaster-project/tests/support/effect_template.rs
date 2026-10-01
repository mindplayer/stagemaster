use serde_json::{Value, json};
use stagemaster_project::{Document, EffectTemplateFile};
pub fn edit(d: &mut Document, command: Value) {
    d.edit(serde_json::from_value(command).unwrap()).unwrap();
}
pub fn setup() -> (Document, String, Vec<String>) {
    let mut d = Document::new("模板绑定").unwrap();
    // Different native slot locations and resolutions, with unrelated static RGB.
    for (name, coarse, fine, address) in [("8 位帕灯", 1, None, 11), ("16 位射灯", 4, Some(2), 101)]
    {
        edit(
            &mut d,
            json!({"op":"fixture","command":{"op":"saveProfile","id":null,"definition":{
                "name":name,"manufacturer":"测试","model":name,"mode":"六通道","footprint":6,
                "channels":[{"attribute":"dimmer","coarse":coarse,"fine":fine,"defaultValue":0},
                {"attribute":"red","coarse":3,"defaultValue":12336},
            {"attribute":"green","coarse":5,"defaultValue":0},
            {"attribute":"blue","coarse":6,"defaultValue":0}]
            }}}),
        );
        let v = d.view();
        edit(
            &mut d,
            json!({"op":"addFixture","name":name,"profileId":v.profiles.last().unwrap().id,
            "domainId":v.domains[0].id,"universe":1,"address":address}),
        );
    }
    edit(&mut d, json!({"op":"addScene","name":"场景"}));
    let v = d.view();
    (
        d,
        v.scenes[0].id.clone(),
        v.fixtures.iter().map(|f| f.id.clone()).collect(),
    )
}
pub fn template() -> EffectTemplateFile {
    EffectTemplateFile::decode(&serde_json::to_vec(&json!({
        "format":"stagemaster-effect-template","formatVersion":1,
        "templateId":"19999999-0000-4000-8000-000000000001",
        "revision":"19999999-0000-4000-8000-000000000002",
        "definition":{"name":"跨型号亮度呼吸","recipe":{
            "kind":"intensity-wave","waveform":"triangle","low":2570,"high":51400,"dutyPercent":50},
            "timing":{"periodMs":1000,"phaseDegrees":0,"spreadDegrees":0,"reverseOrder":false}}
    })).unwrap()).unwrap()
}
pub fn raw(d: &Document) -> Value {
    serde_json::from_slice(&d.encode().unwrap()).unwrap()
}
