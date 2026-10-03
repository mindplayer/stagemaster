use super::{audio, group::id};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn write(path: &Path, plays: &Value) -> PathBuf {
    let manifest = audio::write(path);
    let mut raw: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    raw["requires"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"media.audio-loop-regions","version":1}));
    raw["media"]["audioEditing"]["loopRegions"] = json!([{
        "id":id(99),"name":"红蓝循环","startMs":2000,"endMs":3000,"plays":plays,"enabled":true,"locked":false
    }]);
    fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
    stagemaster_project::Document::decode(&fs::read(path).unwrap()).unwrap();
    manifest
}
pub fn exit(state: &Value, requested: bool) -> Value {
    json!({"kind":"exitLoop","instance":state["audio"]["instance"],"region":state["audio"]["loopState"]["region"],
        "pass":state["audio"]["loopState"]["pass"],"requested":requested})
}
