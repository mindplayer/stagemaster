mod maintenance_support;
use maintenance_support::*;
use stagemaster_install_worker::{Command, Error};
use stagemaster_runtime::{Action, Code, MaintenanceError, Mode, ProgramKey, Status};
use stagemaster_transfer::{Command as WireCommand, Outcome, Request, Upload};

#[test]
fn only_confirmed_maintenance_can_mutate_and_finish_never_starts_playback() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    assert!(matches!(
        worker.process(open_command(1), 0, || Some(epoch(1))).result,
        Err(Error::Maintenance(Code::Mode))
    ));
    assert_eq!(metrics.mutations.get(), 0);
    let quiescence = worker.quiescence_request().unwrap();
    install(&mut worker, &bytes);
    assert!(metrics.mutations.get() > 0);
    let state = worker.finish_maintenance(0).unwrap();
    assert_eq!(state.mode, Mode::Operation);
    assert!(state.bound_package.is_some());
    assert!(state.selected.is_none() && state.loaded.is_none() && state.instance.is_none());
    assert_eq!(metrics.readers.get(), 1);
    assert!(worker.confirm_quiescent(quiescence, 0).is_err());
    let mut frame = [0x55; 512];
    assert!(worker.render(&mut frame).unwrap().is_none());
    assert_eq!(frame, [0x55; 512]);
}

#[test]
fn playing_and_paused_reject_installs_without_stopping_and_release_readers_after_confirmation() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    install(&mut worker, &bytes);
    worker.finish_maintenance(0).unwrap();
    let lease = acquire(&mut worker, 0);
    let e = &worker.catalog()[0];
    let key = ProgramKey {
        kind: e.kind,
        id: e.id,
    };
    apply(&mut worker, lease, Action::Select(key), 0).unwrap();
    apply(&mut worker, lease, Action::Load, 0).unwrap();
    let step = worker.steps()[0].id;
    apply(&mut worker, lease, Action::Start { step }, 0).unwrap();
    let instance = worker.state().instance;
    let before = metrics.mutations.get();
    for now in [1, 2] {
        if now == 2 {
            apply(&mut worker, lease, Action::Pause, now).unwrap();
        }
        assert!(matches!(
            worker
                .process(open_command(2), now, || Some(epoch(2)))
                .result,
            Err(Error::Maintenance(Code::Mode))
        ));
        assert_eq!(worker.state().instance, instance);
        assert_eq!(
            worker.state().status,
            Some(if now == 1 {
                Status::Running
            } else {
                Status::Paused
            })
        );
        assert_eq!(
            apply(&mut worker, lease, Action::BeginMaintenance, now),
            Err(Code::Busy)
        );
    }
    assert_eq!(metrics.mutations.get(), before);
    apply(&mut worker, lease, Action::Stop, 3).unwrap();
    apply(&mut worker, lease, Action::BeginMaintenance, 3).unwrap();
    let cancelled = worker.quiescence_request().unwrap();
    apply(&mut worker, lease, Action::CancelMaintenance, 3).unwrap();
    assert_eq!(worker.confirm_quiescent(cancelled, 3), Err(Code::Mode));
    assert_eq!(metrics.readers.get(), 1);
    apply(&mut worker, lease, Action::BeginMaintenance, 3).unwrap();
    assert!(matches!(
        worker.process(open_command(2), 3, || Some(epoch(2))).result,
        Err(Error::Maintenance(Code::Mode))
    ));
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 3)
        .unwrap();
    assert_eq!(metrics.readers.get(), 0);
    assert!(worker.state().loaded.is_none());
    assert!(worker.confirm_quiescent(cancelled, 3).is_err());
    open(&mut worker, 2, 3);
}

#[test]
fn unfinished_and_failed_transactions_block_exit_until_authoritative_cancel() {
    for fault in [false, true] {
        let (_dir, mut worker, metrics, bytes) = fixture();
        worker
            .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
            .unwrap();
        open(&mut worker, 1, 0);
        metrics.fail_prepare.set(fault);
        let mut upload = Upload::new(bytes.as_slice()).unwrap();
        upload.connect([1; 16]).unwrap();
        for _ in 0..2 {
            let request = upload.outbound().unwrap().unwrap().clone();
            let response = frame(&mut worker, 1, request, 0);
            let _ = upload.accept(response.bytes());
        }
        assert!(matches!(
            worker.finish_maintenance(0),
            Err(MaintenanceError::State(Code::Busy))
        ));
        upload.request_cancel();
        transfer(&mut worker, &mut upload, 1, 0);
        assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
        assert!(
            worker
                .finish_maintenance(0)
                .unwrap()
                .bound_package
                .is_none()
        );
    }
}

#[test]
fn uncertain_commit_requires_reconciliation_and_bad_snapshot_preserves_maintenance() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    open(&mut worker, 1, 0);
    metrics.lose_commit.set(true);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect([1; 16]).unwrap();
    loop {
        let request = upload.outbound().unwrap().unwrap().clone();
        let committing =
            Request::decode(request.bytes()).unwrap().action.command() == WireCommand::Commit;
        let response = frame(&mut worker, 1, request, 0);
        let accepted = upload.accept(response.bytes());
        if committing {
            // Uncertain is an accepted response, but never a terminal outcome.
            // The existing uploader must reconcile it before the maintenance gate opens.
            accepted.unwrap();
            assert_eq!(
                stagemaster_transfer::Response::decode(response.bytes())
                    .unwrap()
                    .result,
                Err(stagemaster_transfer::RemoteError::Uncertain)
            );
            assert_eq!(
                upload.state().unwrap().progress.unwrap().phase,
                stagemaster_install::Phase::Uncertain
            );
            assert!(upload.outcome().is_none());
            break;
        }
        accepted.unwrap();
    }
    assert!(matches!(
        worker.finish_maintenance(0),
        Err(MaintenanceError::State(Code::Busy))
    ));
    upload.retry().unwrap();
    transfer(&mut worker, &mut upload, 1, 0);
    assert!(matches!(upload.outcome(), Some(Outcome::Installed(_))));
    metrics.fail_read.set(true);
    assert!(worker.finish_maintenance(0).is_err());
    assert_eq!(worker.state().mode, Mode::Maintenance);
    assert_eq!(metrics.readers.get(), 0);
    metrics.fail_read.set(false);
    worker.finish_maintenance(0).unwrap();
    assert_eq!(metrics.readers.get(), 1);
}

#[test]
fn ending_maintenance_revokes_old_queue_and_requires_a_fresh_open() {
    let (_dir, mut worker, _metrics, bytes) = fixture();
    install(&mut worker, &bytes);
    worker.finish_maintenance(0).unwrap();
    let lease = acquire(&mut worker, 0);
    apply(&mut worker, lease, Action::BeginMaintenance, 0).unwrap();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    assert!(matches!(
        worker.process(open_command(1), 0, || Some(epoch(1))).result,
        Err(Error::Obsolete)
    ));
    open(&mut worker, 2, 0);
    assert!(matches!(
        worker.process(open_command(1), 0, || Some(epoch(2))).result,
        Err(Error::Obsolete)
    ));
}

#[test]
fn backwards_time_has_no_storage_side_effect_and_invalidates_work_connection() {
    let (_dir, mut worker, metrics, _bytes) = fixture();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 10)
        .unwrap();
    open(&mut worker, 1, 10);
    let state = worker.state();
    let status = Request {
        link: [1; 16],
        id: 1,
        action: stagemaster_transfer::Action::Status,
    }
    .encode()
    .unwrap();
    assert!(matches!(
        worker
            .process(
                Command::Frame {
                    epoch: epoch(1),
                    frame: status.clone()
                },
                9,
                || Some(epoch(1))
            )
            .result,
        Err(Error::Maintenance(Code::Clock))
    ));
    assert_eq!(worker.state(), state);
    assert_eq!(metrics.mutations.get(), 0);
    assert!(matches!(
        worker
            .process(
                Command::Frame {
                    epoch: epoch(1),
                    frame: status
                },
                10,
                || Some(epoch(1))
            )
            .result,
        Err(Error::NotOpen)
    ));
    open(&mut worker, 2, 10);
}

#[test]
fn finished_program_requires_explicit_stop_before_maintenance() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    install(&mut worker, &bytes);
    worker.finish_maintenance(0).unwrap();
    let lease = acquire(&mut worker, 0);
    let entry = worker
        .catalog()
        .iter()
        .find(|p| p.kind == stagemaster_package::Kind::Sequence)
        .unwrap();
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    apply(&mut worker, lease, Action::Select(key), 0).unwrap();
    apply(&mut worker, lease, Action::Load, 0).unwrap();
    let step = worker.steps()[0].id;
    apply(&mut worker, lease, Action::Start { step }, 0).unwrap();
    worker.tick(20_000).unwrap();
    assert_eq!(worker.state().status, Some(Status::Finished));
    let before = metrics.mutations.get();
    assert!(matches!(
        worker
            .process(open_command(2), 20_000, || Some(epoch(2)))
            .result,
        Err(Error::Maintenance(Code::Mode))
    ));
    assert_eq!(metrics.mutations.get(), before);
    assert_eq!(
        apply(&mut worker, lease, Action::BeginMaintenance, 20_000),
        Err(Code::Busy)
    );
    apply(&mut worker, lease, Action::Stop, 20_000).unwrap();
    apply(&mut worker, lease, Action::BeginMaintenance, 20_000).unwrap();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 20_000)
        .unwrap();
    assert_eq!(metrics.readers.get(), 0);
}

#[test]
fn revocation_after_storage_side_effect_does_not_allow_leaving_an_unfinished_transaction() {
    let (_dir, mut worker, metrics, bytes) = fixture();
    worker
        .confirm_quiescent(worker.quiescence_request().unwrap(), 0)
        .unwrap();
    open(&mut worker, 1, 0);
    let mut upload = Upload::new(bytes.as_slice()).unwrap();
    upload.connect([1; 16]).unwrap();
    let status = upload.outbound().unwrap().unwrap().clone();
    upload
        .accept(frame(&mut worker, 1, status, 0).bytes())
        .unwrap();
    let begin = upload.outbound().unwrap().unwrap().clone();
    let mut calls = 0;
    let completion = worker.process(
        Command::Frame {
            epoch: epoch(1),
            frame: begin,
        },
        0,
        || {
            calls += 1;
            if calls == 1 { Some(epoch(1)) } else { None }
        },
    );
    assert!(matches!(completion.result, Err(Error::Obsolete)));
    assert!(metrics.mutations.get() > 0);
    assert!(matches!(
        worker.finish_maintenance(0),
        Err(MaintenanceError::State(Code::Busy))
    ));
    upload.disconnect();
    open(&mut worker, 2, 0);
    upload.connect([2; 16]).unwrap();
    upload.request_cancel();
    transfer(&mut worker, &mut upload, 2, 0);
    assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
    worker.finish_maintenance(0).unwrap();
}
