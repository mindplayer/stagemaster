use crate::support::*;
use serde_json::json;
use stagemaster_live::{
    Command, Session,
    media::{GroupSpec, Limits, Sample},
};
use stagemaster_project::Document;
use stagemaster_time::{Clock, Exchange, Mapping};

pub const GROUP: [u8; 16] = [50; 16];
pub fn provider() -> Clock {
    Clock::new([60; 16], 7).unwrap()
}
pub fn group() -> GroupSpec {
    GroupSpec {
        id: GROUP,
        clock: provider(),
        sources: vec![[1; 16]],
        limits: Limits {
            max_age_ns: 10_000_000,
            max_uncertainty_ns: 2_000_000,
            max_gap_ms: 1000,
            max_rate_percent: 100,
            position_tolerance_ms: 1,
        },
    }
}
pub fn document() -> Document {
    let mut raw = fixture(0);
    raw["lighting"]["scenes"][3]["assignments"] = json!([set("blue", 0)]);
    let mut doc = decode(&raw);
    doc.edit(serde_json::from_value(json!({"op":"effect","command":{"kind":"put","sceneId":id(4),"effect":{
        "id":id(90),"name":"自主呼吸","enabled":true,"fixtureIds":[doc.view().fixtures[0].id],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"blue","low":10_000,"high":50_000}]
    }}})).unwrap()).unwrap();
    doc
}
pub fn setup() -> (Document, Session, Mapping) {
    let doc = document();
    let mut session =
        Session::prepare_with_media(&doc, [9; 16], &specs(), &[group()], 1000).unwrap();
    let map = mapping(session.host_clock(), 200);
    session
        .control(session.key([2; 16]).unwrap(), Command::Execute(0), 1000)
        .unwrap();
    (doc, session, map)
}
pub fn mapping(host: Clock, drift: u32) -> Mapping {
    Mapping::measure(
        Exchange {
            sent: provider().at(10_000_000_000),
            received: provider().at(10_000_000_000),
            received_at_target: host.at(1_000_000_000),
            sent_at_target: host.at(1_000_000_000),
        },
        stagemaster_time::Limits {
            max_round_trip_ns: 1_000_000,
            max_age_ns: 10_000_000_000,
            max_uncertainty_ns: 10_000_000,
            relative_drift_ppm: drift,
            timestamp_error_ns: 0,
        },
    )
    .unwrap()
}
/// Independent forward oscillator model: media clock is 100 ppm faster than host.
pub fn sample(sequence: u64, position: u64, playing: bool, host_ms: u64) -> Sample {
    Sample {
        progress: None,
        at: provider().at(10_000_000_000 + (host_ms - 1000) * 1_000_100),
        sequence,
        position_ms: position,
        playing,
    }
}
pub fn start(doc: &Document, session: &mut Session, map: &Mapping, position: u64, at_ms: u64) {
    let key = session.media_key(GROUP).unwrap();
    let mut prepared = session
        .media_preparer(key)
        .unwrap()
        .prepare(key, doc, position, true, at_ms + 20)
        .unwrap();
    session
        .activate_media(
            &mut prepared,
            sample(1, position, true, at_ms),
            map,
            at_ms + 1,
        )
        .unwrap();
}
#[allow(dead_code)] // Some integration binaries only exercise admission, not frame values.
pub fn assert_frame(doc: &Document, session: &Session, semantic: &[u16]) {
    assert_eq!(session.values().unwrap(), semantic);
    let mut expected = [0; 512];
    doc.compile_scene(&id(1))
        .unwrap()
        .output
        .portable_output()
        .unwrap()
        .render(semantic, &mut expected)
        .unwrap();
    assert_eq!(session.frame().unwrap().slots, expected);
}
