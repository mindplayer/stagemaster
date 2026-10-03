use super::{Harness, group};
use serde_json::{Value, json};
use stagemaster_audio::Resources;
use stagemaster_project::Document;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub fn write(path: &Path) -> PathBuf {
    write_rate(path, 8000)
}
pub fn write_rate(path: &Path, rate: u32) -> PathBuf {
    let manifest = group::write(path);
    let wav = path.with_file_name("music.wav");
    let size = rate * 5 * 2 * 2;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + size).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 4).to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for frame in 0..(rate * 5) {
        let sample = i16::try_from(frame % 8000 + 1).unwrap();
        bytes.extend(sample.to_le_bytes());
        bytes.extend((-sample).to_le_bytes());
    }
    fs::write(&wav, bytes).unwrap();
    let assets = PathBuf::from(format!("{}.assets", path.display()));
    let (digest, _) = Resources::new(assets)
        .import(&wav, "wav", None, &AtomicBool::new(false))
        .unwrap();
    let mut doc = Document::decode(&fs::read(path).unwrap()).unwrap();
    let scenes = doc.view().scenes;
    for command in [
        json!({"kind":"setAsset","asset":{"digest":digest,"fileName":"music.wav","extension":"wav","durationMs":5000}}),
        json!({"kind":"convertLightingClips"}),
        json!({"kind":"addLightingClip","name":"红色","sceneId":scenes[0].id,"startMs":0,"endMs":2500,"fadeMs":0}),
        json!({"kind":"addLightingClip","name":"蓝色","sceneId":scenes[1].id,"startMs":2500,"endMs":5000,"fadeMs":0}),
    ] {
        doc.edit(serde_json::from_value(json!({"op":"audio","command":command})).unwrap())
            .unwrap();
    }
    fs::write(path, doc.encode().unwrap()).unwrap();
    let mut value: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    value["version"] = 2.into();
    value["audio"] = json!({"output":"software"});
    value["sources"][1]["selection"] = json!({"kind":"audioTimeline"});
    fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    manifest
}
pub async fn control(
    h: &Harness,
    session: &str,
    serial: u64,
    state: &Value,
    operation: Value,
) -> Value {
    h.command(session, serial, json!({"kind":"submit","expectedRevision":state["revision"],"action":{
        "kind":"media","group":group::id(2),"generation":state["media"][0]["generation"],"action":operation
    }})).await
}
pub async fn applied(h: &Harness, accepted: &Value) -> Value {
    group::until(h, |s| {
        s["state"]["media"][0]["control"]["status"] == "applied"
            && s["state"]["media"][0]["control"]["request"]
                == accepted["state"]["media"][0]["control"]["request"]
    })
    .await["state"]
        .clone()
}
