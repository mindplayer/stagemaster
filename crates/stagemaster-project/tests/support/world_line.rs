#[allow(dead_code)]
#[path = "position_effect.rs"]
pub mod common;
use serde_json::{Value, json};
use stagemaster_project::Document;

pub fn setup(fine: bool) -> (Document, String, Vec<String>) {
    let (mut doc, scene, ids) = common::setup(fine);
    for (i, id) in ids.iter().enumerate() {
        common::edit(&mut doc,json!({"op":"stage","command":{"op":"putPlacement","placement":{"fixtureId":id,"spaceId":null,"positionMeters":{"x":if i==0 {"-1"}else{"1"},"y":"-2","z":"3"},"rotationDegreesXYZ":{"x":"0","y":"0","z":if i==0 {"0"}else{"30"}}}}})).unwrap();
    }
    common::edit(&mut doc,json!({"op":"position","command":{"op":"calibrate","fixtureId":ids[0],"correction":{"panDegrees":"3","tiltDegrees":"-2"}}})).unwrap();
    (doc, scene, ids)
}
pub fn effect(ids: &[String], fine: bool) -> Value {
    json!({"id":"49999999-0000-4000-8000-000000000001","name":"共同直线","enabled":true,"fixtureIds":ids,
        "periodMs":65536,"spreadDegrees":180,"phaseDegrees":90,"reverse":false,"waveform":"worldLine","dutyPercent":25,
        "channels":[{"attribute":"pan"},{"attribute":"tilt"}],
        "targetPath":{"kind":"line","fromMeters":{"x":"-0.5","y":"1","z":"0.5"},"toMeters":{"x":"0.5","y":"2","z":"0.5"},"branch":"auto","maxErrorMeters":if fine {"0.1"}else{"0.8"}}})
}
