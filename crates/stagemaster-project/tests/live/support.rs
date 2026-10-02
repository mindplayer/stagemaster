use serde_json::{Value, json};
use stagemaster_engine::live::{Handle, Kind, LiveMixer, Source};
use stagemaster_project::{Document, LiveScenePlayer};

pub fn edit(doc: &mut Document, command: Value) {
    doc.edit(serde_json::from_value(command).unwrap()).unwrap();
}
pub fn setup() -> (Document, String, [String; 2]) {
    let mut doc = Document::new("现场贡献验收").unwrap();
    let definition = json!({"name":"混合测试灯","manufacturer":"验收","model":"属性","mode":"8 通道","footprint":8,
        "channels":[
            {"attribute":"dimmer","coarse":1,"fine":null,"defaultValue":12000},
            {"attribute":"pan","coarse":2,"fine":6,"defaultValue":32768},
            {"attribute":"red","coarse":3,"fine":null,"defaultValue":0},
            {"attribute":"color-wheel","coarse":4,"fine":null,"defaultValue":{"functionKey":"white","position":0},
                "functions":[
                    {"key":"white","name":"白色","mode":"slot","dmxFrom":0,"dmxTo":15,"dmxDefault":0},
                    {"key":"red","name":"红色","mode":"slot","dmxFrom":16,"dmxTo":31,"dmxDefault":20},
                    {"key":"blue","name":"蓝色","mode":"slot","dmxFrom":32,"dmxTo":255,"dmxDefault":40}]},
            {"attribute":"green","coarse":7,"fine":null,"defaultValue":0},
            {"attribute":"blue","coarse":8,"fine":null,"defaultValue":0},
            {"attribute":"tilt","coarse":5,"fine":null,"defaultValue":32768}],
        "positioning":{"kind":"intersectingOrthogonal","pan":{"minDegrees":"-270","maxDegrees":"270","reversed":false},"tilt":{"minDegrees":"-135","maxDegrees":"135","reversed":false}}});
    edit(
        &mut doc,
        json!({"op":"fixture","command":{"op":"saveProfile","definition":definition}}),
    );
    let view = doc.view();
    edit(
        &mut doc,
        json!({"op":"addFixture","name":"摇头灯","profileId":view.profiles.last().unwrap().id,"domainId":view.domains[0].id,"universe":1,"address":1}),
    );
    for name in ["底色与运动", "局部亮度"] {
        edit(&mut doc, json!({"op":"addScene","name":name}));
    }
    let view = doc.view();
    let fixture = view.fixtures[0].id.clone();
    let scenes = [view.scenes[0].id.clone(), view.scenes[1].id.clone()];
    // New scene creation stores explicit defaults; remove only the attributes this
    // independent layer is not meant to own, using the real editing command.
    for (index, scene) in scenes.iter().enumerate() {
        for key in [
            "dimmer",
            "pan",
            "red",
            "color-wheel",
            "green",
            "blue",
            "tilt",
        ] {
            if (index == 0 && key == "red") || (index == 1 && key == "pan") {
                continue;
            }
            edit(
                &mut doc,
                json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"mode":"remove","value":0}),
            );
        }
    }
    set(&mut doc, &scenes[0], &fixture, "red", 22_000);
    set(&mut doc, &scenes[1], &fixture, "pan", 0x1fff);
    edit(
        &mut doc,
        json!({"op":"setSceneFunctionValue","sceneId":scenes[0],"fixtureId":fixture,"attribute":"color-wheel","selection":{"functionKey":"red","position":0}}),
    );
    for (index, scene) in scenes.iter().enumerate() {
        let channels = if index == 0 {
            json!([{"attribute":"dimmer","low":10_000,"high":30_000}])
        } else {
            json!([{"attribute":"dimmer","low":20_000,"high":40_000}])
        };
        edit(
            &mut doc,
            json!({"op":"effect","command":{"kind":"put","sceneId":scene,"effect":{
            "id":format!("99999999-0000-4000-8000-00000000000{}",index+1),"name":"动态","enabled":true,"fixtureIds":[fixture],
            "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,"channels":channels}}}),
        );
    }
    edit(
        &mut doc,
        json!({"op":"effect","command":{"kind":"put","sceneId":scenes[0],"effect":{
        "id":"99999999-0000-4000-8000-000000000003","name":"水平运动","enabled":true,"fixtureIds":[fixture],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"position","dutyPercent":50,
        "channels":[{"attribute":"pan","amplitudeDegrees":"30","offsetDegrees":"0","phaseDegrees":0}]}}}),
    );
    (doc, fixture, scenes)
}
pub fn open(m: &mut LiveMixer, id: u8, kind: Kind, priority: i16) -> Handle {
    m.open(Source { id: [id; 16], kind }, priority, m.layout().id())
        .unwrap()
}
pub fn values(m: &LiveMixer) -> [u16; 4] {
    let mut out = [0; 7];
    m.render(&mut out, &mut [None; 7]).unwrap();
    assert_eq!(&out[4..], &[0, 0, 32768]);
    out[..4].try_into().unwrap()
}
pub fn advance(p: &mut LiveScenePlayer, m: &mut LiveMixer, h: Handle, serial: u64, now: u64) {
    p.tick(now).unwrap();
    p.publish(m, h, serial).unwrap();
}
fn set(doc: &mut Document, scene: &str, fixture: &str, key: &str, value: u16) {
    edit(
        doc,
        json!({"op":"setSceneValue","sceneId":scene,"fixtureId":fixture,"attribute":key,"mode":"literal","value":value}),
    );
}
