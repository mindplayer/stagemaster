use super::*;
use crate::media::tests::setup_controlled;
use stagemaster_live::media::Status;
use stagemaster_runtime_host::Backend;

#[test]
fn provider_end_releases_only_its_generation_and_old_end_cannot_stop_a_replacement() {
    let (mut backend, port, prepare, doc, sample, mapping) = setup_controlled(false);
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    let activation = port.stage(plan, sample, mapping).unwrap();
    backend.activate_media(activation, 1000).unwrap();
    port.reclaim(activation, false).unwrap();
    let key = backend.state().media[0].unwrap().group.key;
    let end = port.terminate(key, Termination::Ended).unwrap();
    assert_eq!(port.terminate(key, Termination::Ended), Err(Code::Busy));
    backend.tick(1001).unwrap();
    let ended = backend.state().media[0].unwrap();
    assert_eq!(ended.group.status, Status::Stopped);
    assert_eq!(ended.termination.unwrap().serial, end);
    assert_eq!(ended.termination.unwrap().result, Ok(()));
    assert_eq!(ended.termination.unwrap().generation, key.generation());
    assert_ne!(ended.group.key, key);
    let plan = prepare
        .prepare(ended.group.key, &doc, 0, true, 2000)
        .unwrap();
    let activation = port.stage(plan, sample, mapping).unwrap();
    backend.activate_media(activation, 1002).unwrap();
    port.reclaim(activation, false).unwrap();
    let current = backend.state().media[0].unwrap().group.key;
    let old = port
        .terminate(key, Termination::Failed(Code::Read))
        .unwrap();
    backend.tick(1003).unwrap();
    let retained = backend.state().media[0].unwrap();
    assert_eq!(retained.group.key, current);
    assert_eq!(retained.group.status, Status::Following);
    assert_eq!(retained.termination.unwrap().serial, old);
    assert_eq!(retained.termination.unwrap().result, Err(Code::State));
    assert!(!backend.state().fault);
}

#[test]
fn terminal_slot_is_bounded_nonblocking_and_does_not_wrap_or_survive_shutdown() {
    let (mut backend, port, _, _, _, _) = setup_controlled(false);
    let key = port.initial().key;
    let mut guard = port.inbox.lock().unwrap();
    backend.tick(1000).unwrap();
    assert_eq!(port.terminate(key, Termination::Ended), Err(Code::Busy));
    assert_eq!(backend.observed_ms(), 1000);
    guard.serial = u64::MAX;
    drop(guard);
    assert_eq!(
        port.terminate(key, Termination::Ended),
        Err(Code::Exhausted)
    );
    drop(backend);
    assert_eq!(port.terminate(key, Termination::Ended), Err(Code::State));
}
