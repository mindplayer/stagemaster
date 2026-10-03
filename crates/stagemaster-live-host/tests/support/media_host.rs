#![allow(dead_code)]
use crate::support::fixtures::{decode, fixture, id, set, specs};
use serde_json::json;
use stagemaster_live::{
    Key, Session, SourceSpec,
    media::{GroupSpec, Limits, Preparer, Sample},
};
use stagemaster_live_host::{
    Live, LiveBackend,
    media::{LocalClock, MediaPort},
};
use stagemaster_project::Document;
use stagemaster_runtime_host::{Backend, Configuration, Host};
use stagemaster_time::{Clock, Mapping};
use std::time::{Duration, Instant};

pub struct Rig {
    pub host: Host<Live>,
    pub port: MediaPort,
    pub prepare: Preparer,
    pub doc: Document,
    pub autonomous: Key,
    pub clock: LocalClock,
}
impl Rig {
    pub fn new() -> Self {
        Self::with_backend(|backend| (backend, ())).0
    }
    pub fn with_backend<B: Backend<Profile = Live>, T>(
        make: impl FnOnce(LiveBackend) -> (B, T),
    ) -> (Self, T) {
        let mut raw = fixture(0);
        raw["lighting"]["scenes"][3]["assignments"] = json!([set("blue", 0)]);
        let mut doc = decode(&raw);
        doc.edit(serde_json::from_value(json!({"op":"effect","command":{"kind":"put","sceneId":id(4),"effect":{
            "id":id(90),"name":"独立呼吸","enabled":true,"fixtureIds":[doc.view().fixtures[0].id],
            "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
            "channels":[{"attribute":"blue","low":10_000,"high":50_000}]
        }}})).unwrap()).unwrap();
        Self::from_prepared_document(doc, &specs(), make)
    }
    pub fn with_document(doc: Document, sources: &[SourceSpec]) -> Self {
        Self::from_prepared_document(doc, sources, |backend| (backend, ())).0
    }
    fn from_prepared_document<B: Backend<Profile = Live>, T>(
        doc: Document,
        sources: &[SourceSpec],
        make: impl FnOnce(LiveBackend) -> (B, T),
    ) -> (Self, T) {
        let provider = Clock::new([60; 16], 1).unwrap();
        let group = GroupSpec {
            id: [50; 16],
            clock: provider,
            sources: vec![[1; 16]],
            limits: Limits {
                max_age_ns: 500_000_000,
                max_uncertainty_ns: 1_000_000,
                max_gap_ms: 500,
                max_rate_percent: 400,
                position_tolerance_ms: 1,
            },
        };
        let session = Session::prepare_with_media(&doc, [9; 16], sources, &[group], 0).unwrap();
        let target = session.host_clock();
        let key = session.media_key([50; 16]).unwrap();
        let prepare = session.media_preparer(key).unwrap();
        let autonomous = session.key([2; 16]).unwrap();
        let (backend, mut ports) = LiveBackend::with_media(session).unwrap();
        let (backend, extra) = make(backend);
        let host = Host::start_backend(
            backend,
            Configuration {
                period: Duration::from_millis(5),
            },
        )
        .unwrap();
        let clock = LocalClock::new(host.clock(), provider, target, 5_000_000_000).unwrap();
        (
            Self {
                host,
                port: ports.remove(0),
                prepare,
                doc,
                autonomous,
                clock,
            },
            extra,
        )
    }
    pub fn sample(
        &self,
        seq: u64,
        pos: u64,
        playing: bool,
        original: Instant,
    ) -> (Sample, Mapping) {
        let (at, map) = self.clock.map(original).unwrap();
        (
            Sample {
                at,
                sequence: seq,
                position_ms: pos,
                playing,
            },
            map,
        )
    }
    pub fn reached(&self, sample: Sample, map: &Mapping) {
        let nanos = map.convert(sample.at).unwrap().latest().nanos;
        // The actual host exposes integer milliseconds. Wait for its frontier, never restamp the sample.
        crate::support::until(&self.host.observer(), |s| {
            s.state.observed_ms * 1_000_000 >= nanos
        });
    }
    pub fn assert_light(&self, frame: &[u8; 512], values: [u16; 3]) {
        let mut expected = [0; 512];
        self.doc
            .compile_scene(&id(1))
            .unwrap()
            .output
            .portable_output()
            .unwrap()
            .render(&[values[0], values[1], values[2], 0], &mut expected)
            .unwrap();
        // The independent fourth channel is a running blue effect, not a fixed timeline value.
        assert_eq!(&frame[..3], &expected[..3]);
    }
}
