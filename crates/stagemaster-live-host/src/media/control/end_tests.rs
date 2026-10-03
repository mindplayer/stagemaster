use super::{
    tests::{lease, submit},
    *,
};
use crate::media::tests::setup_controlled;
use stagemaster_live::media::Status;
use stagemaster_runtime_host::Backend;

fn end(playing: bool) -> MediaCommand {
    MediaCommand::Seek {
        position_ms: 1000,
        playing,
    }
}

#[test]
fn end_seek_requires_actual_completion_and_releases_once_without_a_replacement_plan() {
    for playing in [false, true] {
        let (mut backend, port, prepare, doc, sample, map) = setup_controlled(true);
        let lease = lease(&mut backend);
        let start = submit(&mut backend, lease, 1, MediaCommand::Play, 1000);
        let request = start.state.media[0].unwrap().control.unwrap().request;
        let plan = prepare
            .prepare(port.initial().key, &doc, 0, true, 2000)
            .unwrap();
        let staged = port
            .stage_requested(request.ticket, plan, sample, map)
            .unwrap();
        backend.tick(1001).unwrap();
        port.reclaim(staged, false).unwrap();
        port.complete_control(request.ticket, Ok(())).unwrap();
        backend.tick(1002).unwrap();
        let active = backend.state().media[0].unwrap().group;
        assert_eq!(active.status, Status::Following);
        let accepted = submit(&mut backend, lease, 2, end(playing), 1003);
        let request = accepted.state.media[0].unwrap().control.unwrap().request;
        backend.tick(1004).unwrap();
        assert_eq!(backend.state().media[0].unwrap().group, active);
        assert_eq!(
            backend.state().media[0].unwrap().control.unwrap().result,
            None
        );
        port.complete_control(request.ticket, Ok(())).unwrap();
        backend.tick(1005).unwrap();
        let ended = backend.state().media[0].unwrap();
        assert_eq!(ended.control.unwrap().result, Some(Ok(())));
        assert_eq!(ended.group.status, Status::Stopped);
        assert_eq!(ended.group.key.generation(), active.key.generation() + 1);
        assert!(ended.termination.is_none());
        assert_eq!(
            port.complete_control(request.ticket, Ok(())),
            Err(Code::State)
        );
        backend.tick(1006).unwrap();
        assert_eq!(backend.state().media[0].unwrap().group, ended.group);
    }
}

#[test]
fn failed_expired_and_superseded_end_requests_do_not_release_the_group() {
    for expired in [false, true] {
        let (mut backend, port, _, _, _, _) = setup_controlled(true);
        let lease = lease(&mut backend);
        let before = backend.state().media[0].unwrap().group;
        let accepted = submit(&mut backend, lease, 1, end(true), 1000);
        let request = accepted.state.media[0].unwrap().control.unwrap().request;
        port.complete_control(
            request.ticket,
            if expired { Ok(()) } else { Err(Code::Read) },
        )
        .unwrap();
        backend.tick(if expired { 1050 } else { 1001 }).unwrap();
        let state = backend.state().media[0].unwrap();
        assert_eq!(state.group, before);
        assert_eq!(
            state.control.unwrap().result,
            Some(Err(if expired {
                ControlFailure::TimedOut
            } else {
                ControlFailure::Provider(Code::Read)
            }))
        );
    }
    let (mut backend, port, prepare, doc, sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let accepted = submit(&mut backend, lease, 1, end(true), 1000);
    let stale = accepted.state.media[0].unwrap().control.unwrap().request;
    let next = submit(&mut backend, lease, 2, MediaCommand::Play, 1001);
    let current = next.state.media[0].unwrap().control.unwrap().request;
    assert_eq!(
        port.complete_control(stale.ticket, Ok(())),
        Err(Code::State)
    );
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    let staged = port
        .stage_requested(current.ticket, plan, sample, map)
        .unwrap();
    backend.tick(1002).unwrap();
    port.reclaim(staged, false).unwrap();
    port.complete_control(current.ticket, Ok(())).unwrap();
    backend.tick(1003).unwrap();
    assert_eq!(
        port.complete_control(stale.ticket, Ok(())),
        Err(Code::State)
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Following
    );
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Ok(()))
    );
}

#[test]
fn end_confirmation_cannot_release_a_group_replaced_by_provider_termination() {
    let (mut backend, port, _, _, _, _) = setup_controlled(true);
    let lease = lease(&mut backend);
    let accepted = submit(&mut backend, lease, 1, end(false), 1000);
    let request = accepted.state.media[0].unwrap().control.unwrap().request;
    port.terminate(port.initial().key, crate::media::Termination::Ended)
        .unwrap();
    backend.tick(1001).unwrap();
    let stopped = backend.state().media[0].unwrap().group;
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(1002).unwrap();
    assert_eq!(backend.state().media[0].unwrap().group, stopped);
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
}

#[test]
fn a_staged_end_plan_is_rejected_before_it_can_change_lighting() {
    let (mut backend, port, prepare, doc, mut sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let before = backend.state().media[0].unwrap().group;
    let accepted = submit(&mut backend, lease, 1, end(false), 1000);
    let request = accepted.state.media[0].unwrap().control.unwrap().request;
    let plan = prepare
        .prepare(before.key, &doc, 1000, false, 2000)
        .unwrap();
    sample.position_ms = 1000;
    sample.playing = false;
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    backend.tick(1001).unwrap();
    assert_eq!(
        port.reclaim(staged, false).unwrap().result,
        Some(Err(Code::State))
    );
    assert_eq!(backend.state().media[0].unwrap().group, before);
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
}
