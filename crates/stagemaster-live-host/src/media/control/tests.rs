use super::*;
use crate::{Action, LiveBackend, State, media::tests::setup_controlled};
use stagemaster_live::media::Status;
use stagemaster_runtime::{Grant, Lease, Origin, Request};
use stagemaster_runtime_host::Backend;

pub(super) fn lease(backend: &mut LiveBackend) -> Lease {
    backend
        .acquire(
            Grant {
                principal: [1; 16],
                origin: Origin::Remote,
                duration_ms: 1000,
            },
            false,
            1000,
        )
        .unwrap()
}
pub(super) fn submit(
    backend: &mut LiveBackend,
    lease: Lease,
    serial: u64,
    command: MediaCommand,
    now: u64,
) -> stagemaster_runtime::Receipt<Action, State> {
    backend
        .submit(
            Request {
                lease,
                serial,
                expected_revision: backend.state().revision,
                action: Action::RequestMedia {
                    group: backend.state().media[0].unwrap().group.key,
                    command,
                },
            },
            now,
        )
        .unwrap()
}

#[test]
fn staged_old_request_cannot_activate_after_a_new_intent_and_plans_remain_reclaimable() {
    let (mut backend, port, prepare, doc, sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let first = submit(&mut backend, lease, 1, MediaCommand::Play, 1000);
    let request = first.state.media[0].unwrap().control.unwrap().request;
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    let guard = port.staging.lock().unwrap(); // Worker must stay responsive while staging is busy.
    let second = submit(&mut backend, lease, 2, MediaCommand::Stop, 1001);
    assert_eq!(second.result, Ok(()));
    assert_eq!(second.state.media[0].unwrap().group.status, Status::Ready);
    drop(guard);
    backend.tick(1002).unwrap();
    assert_eq!(
        port.reclaim(staged, false).unwrap().result,
        Some(Err(Code::State))
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Ready
    );
    assert_eq!(
        port.complete_control(request.ticket, Ok(())),
        Err(Code::State)
    );
    let stop = second.state.media[0].unwrap().control.unwrap().request;
    port.complete_control(stop.ticket, Ok(())).unwrap();
    backend.tick(1003).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Ok(()))
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Stopped
    );
}

#[test]
fn expired_preparation_or_completion_does_not_change_group_and_never_blocks_tick() {
    let (mut backend, port, prepare, doc, sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let first = submit(&mut backend, lease, 1, MediaCommand::Play, 1000);
    let request = first.state.media[0].unwrap().control.unwrap().request;
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    let guard = port.staging.lock().unwrap();
    let control = port.control.as_ref().unwrap().lock().unwrap();
    backend.tick(1051).unwrap();
    assert_eq!(backend.observed_ms(), 1051);
    drop(control);
    port.complete_control(request.ticket, Ok(())).unwrap(); // Queued completion cannot defeat deadline.
    backend.tick(1052).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::TimedOut))
    );
    drop(guard);
    backend.tick(1053).unwrap();
    assert_eq!(
        port.reclaim(staged, false).unwrap().result,
        Some(Err(Code::State))
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Ready
    );
    drop(backend);
    assert_eq!(port.control_state(), Err(Code::State));
    assert_eq!(
        port.complete_control(request.ticket, Ok(())),
        Err(Code::State)
    );
}

#[test]
fn busy_exhausted_failed_and_unconfirmed_controls_have_distinct_outcomes() {
    let (mut backend, port, _, _, _, _) = setup_controlled(true);
    let lease = lease(&mut backend);
    let guard = port.control.as_ref().unwrap().lock().unwrap();
    assert_eq!(
        submit(&mut backend, lease, 1, MediaCommand::Play, 1000).result,
        Err(Code::Busy)
    );
    drop(guard);
    let first = submit(&mut backend, lease, 2, MediaCommand::Pause, 1001);
    let request = first.state.media[0].unwrap().control.unwrap().request;
    port.complete_control(request.ticket, Ok(())).unwrap();
    assert_eq!(
        port.complete_control(request.ticket, Ok(())),
        Err(Code::Busy)
    );
    backend.tick(1002).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
    let next = submit(&mut backend, lease, 3, MediaCommand::Play, 1003);
    let request = next.state.media[0].unwrap().control.unwrap().request;
    port.complete_control(request.ticket, Err(Code::Read))
        .unwrap();
    backend.tick(1004).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::Read)))
    );
    port.control.as_ref().unwrap().lock().unwrap().serial = u64::MAX;
    assert_eq!(
        submit(&mut backend, lease, 4, MediaCommand::Play, 1005).result,
        Err(Code::Exhausted)
    );
    assert_eq!(port.control_state().unwrap().unwrap().request, request);
}

#[test]
fn seek_requires_its_own_activation_and_old_group_commands_cannot_replace_the_intent() {
    let (mut backend, port, prepare, doc, sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let first = submit(&mut backend, lease, 1, MediaCommand::Play, 1000);
    let request = first.state.media[0].unwrap().control.unwrap().request;
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    backend.tick(1001).unwrap();
    assert_eq!(port.reclaim(staged, false).unwrap().result, Some(Ok(())));
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(1002).unwrap();
    let seek = submit(
        &mut backend,
        lease,
        2,
        MediaCommand::Seek {
            position_ms: 0,
            playing: true,
        },
        1003,
    );
    let request = seek.state.media[0].unwrap().control.unwrap().request;
    assert_eq!(seek.state.media[0].unwrap().group.status, Status::Following);
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(1004).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
    let stale = backend
        .submit(
            Request {
                lease,
                serial: 3,
                expected_revision: backend.state().revision,
                action: Action::RequestMedia {
                    group: port.initial().key,
                    command: MediaCommand::Stop,
                },
            },
            1005,
        )
        .unwrap();
    assert_eq!(stale.result, Err(Code::Selection));
    assert_eq!(port.control_state().unwrap().unwrap().request, request);
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Following
    );
}

#[test]
fn rejected_prepared_position_reports_failure_without_changing_execution() {
    let (mut backend, port, prepare, doc, mut sample, map) = setup_controlled(true);
    let lease = lease(&mut backend);
    let first = submit(&mut backend, lease, 1, MediaCommand::Play, 1000);
    let request = first.state.media[0].unwrap().control.unwrap().request;
    let plan = prepare
        .prepare(port.initial().key, &doc, 0, true, 2000)
        .unwrap();
    sample.position_ms = 1; // Real admission must reject a sample that disagrees with its preparation.
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    backend.tick(1001).unwrap();
    assert_eq!(
        port.reclaim(staged, false).unwrap().result,
        Some(Err(Code::State))
    );
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.key,
        port.initial().key
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.status,
        Status::Ready
    );
    assert!(!backend.state().fault);
}
