use serde_json::{Value, json};
use stagemaster_playback::Player;
use stagemaster_project::{AudioLightingClip, Document};
pub fn audio(doc: &mut Document, command: Value) -> Result<(), String> {
    doc.edit(stagemaster_project::EditCommand::Audio {
        command: serde_json::from_value(command).unwrap(),
    })
}
pub fn fixture() -> Document {
    let mut d = Document::new("保相位分割").unwrap();
    let v = d.view();
    d.edit(serde_json::from_value(json!({"op":"addFixture","name":"灯","profileId":v.profiles[0].id,"domainId":v.domains[0].id,"universe":1,"address":1})).unwrap()).unwrap();
    for name in ["动态", "暗场"] {
        d.edit(serde_json::from_value(json!({"op":"addScene","name":name})).unwrap())
            .unwrap();
    }
    let v = d.view();
    let a = &v.scenes[0].id;
    let b = &v.scenes[1].id;
    d.edit(serde_json::from_value(json!({"op":"effect","command":{"kind":"put","sceneId":a,"effect":{"id":"b0000000-0000-4000-8000-000000000001","name":"运动","enabled":true,"fixtureIds":[v.fixtures[0].id],"periodMs":997,"spreadDegrees":0,"phaseDegrees":19,"reverse":false,"waveform":"triangle","dutyPercent":50,"channels":[{"attribute":"dimmer","low":1000,"high":60000}]}}})).unwrap()).unwrap();
    audio(&mut d,json!({"kind":"setAsset","asset":{"digest":"ab".repeat(32),"fileName":"曲.wav","extension":"wav","durationMs":10000}})).unwrap();
    audio(&mut d, json!({"kind":"convertLightingClips"})).unwrap();
    for (start, end, scene, fade) in [(0, 4000, a, 500), (4000, 6000, b, 750)] {
        audio(&mut d,json!({"kind":"addLightingClip","name":"段","sceneId":scene,"startMs":start,"endMs":end,"fadeMs":fade})).unwrap();
    }
    d
}
pub fn clips(d: &Document) -> Vec<AudioLightingClip> {
    d.audio_timeline().unwrap().lighting_clips.unwrap()
}
pub fn split(d: &mut Document, id: &str, time: u64) -> Result<(), String> {
    audio(d, json!({"kind":"splitLightingClip","id":id,"timeMs":time}))
}
pub fn sample(d: &Document, time: u64) -> Vec<u16> {
    let t = d.audio_timeline().unwrap();
    let active = t.lighting_at(time);
    let compiled = d.compile_audio_lighting(active.map(|c| c.id)).unwrap();
    let mut p = Player::new(compiled.plan, 0);
    p.execute(0, 0).unwrap();
    p.advance(time - active.map_or(0, |c| c.start_ms)).unwrap();
    p.values().to_vec()
}
