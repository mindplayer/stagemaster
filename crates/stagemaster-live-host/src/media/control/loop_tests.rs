use super::{
    tests::{lease, submit},
    *,
};
use crate::{media::Termination, media::tests::setup_controlled};
use stagemaster_live::media::Status;
use stagemaster_runtime_host::Backend;

#[test]
fn rejected_or_expired_exit_keeps_playing_and_late_completion_cannot_apply_after_termination() {
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
    let key = backend.state().media[0].unwrap().group.key;
    let command = MediaCommand::ExitLoop {
        instance: 1,
        region: 0,
        pass: 1,
        requested: true,
    };
    let exit = submit(&mut backend, lease, 2, command, 1003);
    let request = exit.state.media[0].unwrap().control.unwrap().request;
    port.complete_control(request.ticket, Err(Code::Selection))
        .unwrap();
    backend.tick(1004).unwrap();
    let media = backend.state().media[0].unwrap();
    assert_eq!(media.group.key, key);
    assert_eq!(media.group.status, Status::Following);
    assert_eq!(
        media.control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::Selection)))
    );
    let retry = submit(&mut backend, lease, 3, command, 1005);
    let request = retry.state.media[0].unwrap().control.unwrap().request;
    backend.tick(request.deadline_ms).unwrap();
    let media = backend.state().media[0].unwrap();
    assert_eq!(media.group.status, Status::Following);
    assert_eq!(media.group.key, key);
    assert_eq!(
        media.control.unwrap().result,
        Some(Err(ControlFailure::TimedOut))
    );
    assert_eq!(
        port.complete_control(request.ticket, Ok(())),
        Err(Code::State)
    );
    let now = request.deadline_ms + 1;
    let retry = submit(&mut backend, lease, 4, command, now);
    let request = retry.state.media[0].unwrap().control.unwrap().request;
    port.terminate(key, Termination::Ended).unwrap();
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(now + 1).unwrap();
    let media = backend.state().media[0].unwrap();
    assert_eq!(media.group.status, Status::Stopped);
    assert_eq!(
        media.control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
}
