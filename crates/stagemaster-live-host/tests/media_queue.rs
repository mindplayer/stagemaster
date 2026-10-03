#[path = "support/gate.rs"]
mod gate;
#[path = "support/media_host.rs"]
mod media_support;
mod support;
use gate::Delayed;
use media_support::Rig;
use stagemaster_live::Command;
use stagemaster_live_host::Action;
use stagemaster_runtime::Code;
use stagemaster_runtime_host::{Error, Phase, QUEUE_CAPACITY};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use support::*;

#[test]
fn saturated_expired_activation_queue_keeps_one_prepared_plan_and_retry_uses_original_authority() {
    let (mut rig, gate) = Rig::with_backend(Delayed::new);
    let client = connect(&rig.host, 1, false, 60_000);
    let state = send(
        &client,
        1,
        client.acquired_state().revision,
        control(rig.autonomous, Command::Execute(0)),
    );
    let key = rig.port.initial().key;
    let (sample, map) = rig.sample(1, 0, true, Instant::now());
    rig.reached(sample, &map);
    let prepared = rig
        .prepare
        .prepare(key, &rig.doc, 0, true, state.observed_ms + 2000)
        .unwrap();
    let ticket = rig.port.stage(prepared, sample, map).unwrap();
    gate.armed.store(true, Ordering::SeqCst);
    gate.entered.recv_timeout(WAIT).unwrap();
    let mut waits = Vec::new();
    for _ in 0..QUEUE_CAPACITY {
        waits.push(
            client
                .submit(
                    2,
                    state.revision,
                    Action::ActivateMedia { ticket },
                    Duration::from_millis(1),
                )
                .unwrap(),
        );
    }
    assert!(matches!(
        client.submit(2, state.revision, Action::ActivateMedia { ticket }, TTL),
        Err(Error::QueueFull)
    ));
    std::thread::sleep(Duration::from_millis(5));
    gate.release.send(()).unwrap();
    for wait in waits {
        assert_eq!(wait.wait(WAIT).unwrap(), Err(Error::Deadline));
    }
    assert!(matches!(rig.port.reclaim(ticket, false), Err(Code::Busy)));
    let observed = until(&rig.host.observer(), |s| {
        s.state.observed_ms > state.observed_ms
    });
    assert_eq!(observed.state.owner.unwrap().serial, 1);
    assert_eq!(observed.state.media[0].unwrap().group.key, key);
    let activated = send(&client, 2, state.revision, Action::ActivateMedia { ticket });
    assert_ne!(activated.media[0].unwrap().group.key, key);
    assert_eq!(
        rig.port.reclaim(ticket, false).unwrap().result,
        Some(Ok(()))
    );
    let next_key = activated.media[0].unwrap().group.key;
    let prepared = rig
        .prepare
        .prepare(next_key, &rig.doc, 0, true, activated.observed_ms + 2000)
        .unwrap();
    let pending = rig.port.stage(prepared, sample, map).unwrap();
    gate.fail.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + WAIT;
    while rig.host.observer().phase() != Phase::Faulted {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(rig.host.shutdown(WAIT).unwrap(), Phase::Faulted);
    assert!(rig.host.observer().read().unwrap().snapshot.is_none());
    assert_eq!(rig.port.publish(next_key, sample, map), Err(Code::State));
    assert_eq!(rig.port.reclaim(pending, true).unwrap().result, None);
}
