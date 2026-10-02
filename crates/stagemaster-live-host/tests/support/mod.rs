#![allow(dead_code)]
#[path = "../../../stagemaster-live/tests/support/mod.rs"]
mod fixtures;
use fixtures::{decode, fixture, id, playback, specs};
use stagemaster_live::{Command, Key, Session};
use stagemaster_live_host::{Action, Live, LiveBackend, State};
use stagemaster_project::PackageSelection;
use stagemaster_runtime::{Grant, Origin};
use stagemaster_runtime_host::{Client, Configuration, Error, Host, Observer, Snapshot};
use std::time::{Duration, Instant};

pub const TTL: Duration = Duration::from_secs(1);
pub const WAIT: Duration = Duration::from_secs(3);
pub fn prepared() -> (LiveBackend, [Key; 4]) {
    let mut doc = decode(&fixture(70));
    doc.edit(serde_json::from_value(serde_json::json!({"op":"effect","command":{"kind":"put","sceneId":id(5),"effect":{
        "id":id(90),"name":"呼吸","enabled":true,"fixtureIds":[doc.view().fixtures[0].id],
        "periodMs":1000,"spreadDegrees":0,"phaseDegrees":0,"reverse":false,"waveform":"triangle","dutyPercent":50,
        "channels":[{"attribute":"dimmer","low":10_000,"high":50_000}]
    }}})).unwrap()).unwrap();
    let mut specs = specs();
    specs.push(playback(4, PackageSelection::Scene { id: id(5) }));
    let session = Session::prepare(&doc, [9; 16], &specs, 0).unwrap();
    let keys = [1, 2, 3, 4].map(|n| session.key([n; 16]).unwrap());
    (LiveBackend::new(session).unwrap(), keys)
}
pub fn start() -> (Host<Live>, [Key; 4]) {
    let (backend, keys) = prepared();
    (
        Host::start_backend(
            backend,
            Configuration {
                period: Duration::from_millis(5),
            },
        )
        .unwrap(),
        keys,
    )
}
pub fn grant(n: u8, duration_ms: u64) -> Grant {
    Grant {
        principal: [n; 16],
        origin: Origin::Remote,
        duration_ms,
    }
}
pub fn connect(host: &Host<Live>, n: u8, takeover: bool, duration: u64) -> Client<Live> {
    host.connect(grant(n, duration), takeover, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap()
}
pub fn until(
    observer: &Observer<Live>,
    predicate: impl Fn(&Snapshot<Live>) -> bool,
) -> Snapshot<Live> {
    let end = Instant::now() + WAIT;
    loop {
        match observer.read() {
            Ok(o) => {
                if let Some(s) = o.snapshot
                    && predicate(&s)
                {
                    return s;
                }
            }
            Err(Error::ObservationBusy) => {}
            Err(e) => panic!("{e}"),
        }
        assert!(Instant::now() < end, "未观察到期望状态");
        std::thread::sleep(Duration::from_millis(2));
    }
}
pub fn control(key: Key, command: Command) -> Action {
    Action::Control {
        source: key,
        command,
    }
}
pub fn send(client: &Client<Live>, serial: u64, revision: u64, action: Action) -> State {
    let r = client
        .submit(serial, revision, action, TTL)
        .unwrap()
        .wait(WAIT)
        .unwrap()
        .unwrap();
    r.result.unwrap();
    r.state
}
