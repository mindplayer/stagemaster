mod support;
use stagemaster_package::{Archive, MAX_LOADER_BYTES};
use stagemaster_playback::Player;
use stagemaster_runtime::*;
use std::cell::Cell;
use support::*;

#[test]
fn all_exported_programs_replay_identically_with_no_reader_access_after_loading() {
    for repeat in [false, true] {
        let fixture = Fixture::new(repeat);
        let archive = Archive::open(fixture.bytes.as_slice()).unwrap();
        for index in 0..archive.entries().len() {
            let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
            let lease = acquire(&mut runtime, Origin::Panel, 0);
            let key = load(&mut runtime, lease, index, 0);
            let expected = archive.load(fixture.bytes.as_slice(), index).unwrap();
            let mut reference = Player::new(expected.plan, 0);
            start(&mut runtime, lease, 0);
            reference.execute(0, 0).unwrap();
            fixture.metrics.reads.set(0);
            fixture.metrics.fail.set(true);
            for now in (0..30_000).step_by(25) {
                runtime.tick(now).unwrap();
                reference.advance(now).unwrap();
                let mut actual = [0; 512];
                let mut wanted = [0; 512];
                let frame = runtime.render(&mut actual).unwrap().unwrap();
                expected
                    .output
                    .render(reference.values(), &mut wanted)
                    .unwrap();
                assert_eq!(actual, wanted);
                assert_eq!(runtime.state().status, Some(reference.status()));
                assert_eq!(frame.program, key);
                assert_eq!(frame.sampled_ms, now);
            }
            assert_eq!(fixture.metrics.reads.get(), 0);
            fixture.metrics.fail.set(false);
            apply(&mut runtime, lease, Action::Stop, 30000);
            let mut defaults = [0; 512];
            runtime.render(&mut defaults).unwrap();
            assert_ne!(defaults[0], 0, "stop preserves fixture defaults");
            assert!(runtime.state().instance.is_none());
        }
    }
}
#[test]
fn duplicate_commands_return_historical_receipts_without_new_instances_or_policy_calls() {
    let fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Remote, 0);
    load(&mut runtime, lease, 2, 0);
    let request = request(
        &runtime,
        lease,
        Action::Start {
            step: runtime.steps()[0].id,
        },
    );
    let receipt = runtime.submit(request, 0).unwrap();
    let instance = runtime.state().instance;
    let repeated = runtime.submit(request, 250).unwrap();
    assert_eq!(repeated, receipt);
    assert_eq!(runtime.state().instance, instance);
    assert_eq!(runtime.state().elapsed_ms, 250);
    assert_eq!(fixture.policy.0.borrow().permissions.len(), 1);
    let state = runtime.state();
    let conflict = Request {
        action: Action::Stop,
        ..request
    };
    assert_eq!(runtime.submit(conflict, 250), Err(Code::Sequence));
    assert!(runtime.state().owner.is_none());
    assert_eq!(runtime.state().instance, state.instance);
    assert_eq!(runtime.state().status, Some(Status::Running));
}
#[test]
fn control_takeover_expiry_disconnect_and_reconnect_never_change_playback_intent() {
    let fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let remote = acquire(&mut runtime, Origin::Remote, 0);
    load(&mut runtime, remote, 2, 0);
    start(&mut runtime, remote, 0);
    let old = request(&runtime, remote, Action::Stop);
    let instance = runtime.state().instance;
    assert_eq!(
        runtime.acquire(grant(Origin::Panel, 100), false, 10),
        Err(Code::Busy)
    );
    let panel = runtime
        .acquire(grant(Origin::Panel, 100), true, 10)
        .unwrap();
    assert_eq!(runtime.submit(old, 20), Err(Code::Lease));
    assert_eq!(runtime.release(remote, 20), Err(Code::Lease));
    assert_eq!(runtime.state().owner.unwrap().lease, panel);
    assert_eq!(runtime.state().instance, instance);
    runtime.tick(110).unwrap();
    assert!(runtime.state().owner.is_none());
    assert_eq!(runtime.state().status, Some(Status::Running));
    assert_eq!(runtime.renew(panel, 100, 110), Err(Code::Lease));
    let new = acquire(&mut runtime, Origin::Remote, 200);
    assert_ne!(new, remote);
    assert_eq!(runtime.state().instance, instance);
    runtime.release(new, 210).unwrap();
    runtime.tick(10_000).unwrap();
    assert_eq!(runtime.state().status, Some(Status::Running));
    let stopped = acquire(&mut runtime, Origin::Panel, 10_000);
    apply(&mut runtime, stopped, Action::Stop, 10_000);
}
#[test]
fn selection_is_separate_from_running_and_invalid_selection_preserves_context() {
    let fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    let playing = load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 0);
    let instance = runtime.state().instance;
    let choice = ProgramKey {
        kind: runtime.catalog()[0].kind,
        id: runtime.catalog()[0].id,
    };
    apply(&mut runtime, lease, Action::Select(choice), 100);
    assert_eq!(runtime.state().loaded, Some(playing));
    assert_eq!(runtime.state().instance, instance);
    assert_eq!(
        submit(&mut runtime, lease, Action::Load, 100).result,
        Err(Code::Busy)
    );
    let step = runtime.steps()[0].id;
    assert_eq!(
        submit(&mut runtime, lease, Action::Start { step }, 100).result,
        Err(Code::NotLoaded)
    );
    let unknown = ProgramKey {
        id: [255; 16],
        ..choice
    };
    assert_eq!(
        submit(&mut runtime, lease, Action::Select(unknown), 100).result,
        Err(Code::Selection)
    );
    assert_eq!(runtime.state().selected, Some(choice));
    apply(&mut runtime, lease, Action::Stop, 100);
    apply(&mut runtime, lease, Action::Load, 100);
    assert_eq!(runtime.state().loaded, Some(choice));
    assert_eq!(runtime.state().status, Some(Status::Idle));
}
#[test]
fn failed_load_is_recoverable_and_does_not_claim_to_preserve_a_released_plan() {
    let fixture = Fixture::new(false);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    load(&mut runtime, lease, 0, 0);
    let entry = &runtime.catalog()[1];
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    apply(&mut runtime, lease, Action::Select(key), 0);
    fixture.metrics.fail.set(true);
    let request = request(&runtime, lease, Action::Load);
    let receipt = runtime.submit(request, 0).unwrap();
    assert_eq!(receipt.result, Err(Code::Read));
    assert_eq!(runtime.state().selected, Some(key));
    assert!(runtime.state().loaded.is_none());
    let mut untouched = [0x99; 512];
    assert_eq!(runtime.render(&mut untouched).unwrap(), None);
    assert_eq!(untouched, [0x99; 512]);
    fixture.metrics.fail.set(false);
    let reads = fixture.metrics.reads.get();
    assert_eq!(runtime.submit(request, 10).unwrap(), receipt);
    assert_eq!(fixture.metrics.reads.get(), reads);
    apply(&mut runtime, lease, Action::Load, 10);
    assert_eq!(runtime.state().loaded, Some(key));
}
#[test]
fn preflight_budget_refusal_preserves_the_existing_plan() {
    let fixture = Fixture::new(false);
    let source = fixture.snapshot();
    let entries = source.archive().entries();
    let small = entries[0].usage.loader_peak_bytes;
    assert!(entries[2].usage.loader_peak_bytes > small);
    drop(source);
    let mut runtime = fixture.runtime(small);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    let first = load(&mut runtime, lease, 0, 0);
    let entry = &runtime.catalog()[2];
    let key = ProgramKey {
        kind: entry.kind,
        id: entry.id,
    };
    apply(&mut runtime, lease, Action::Select(key), 0);
    assert_eq!(
        submit(&mut runtime, lease, Action::Load, 0).result,
        Err(Code::Budget)
    );
    assert_eq!(runtime.state().loaded, Some(first));
}
#[test]
fn paused_resume_manual_next_and_permission_checks_preserve_instance_identity() {
    let fixture = Fixture::new(false);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Remote, 0);
    load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 0);
    let instance = runtime.state().instance;
    apply(&mut runtime, lease, Action::Pause, 100);
    runtime.tick(300).unwrap();
    assert_eq!(runtime.state().elapsed_ms, 100);
    fixture.policy.0.borrow_mut().deny = Some(Denial::Expired);
    assert_eq!(
        submit(&mut runtime, lease, Action::Resume, 300).result,
        Err(Code::Permission(Denial::Expired))
    );
    assert_eq!(runtime.state().status, Some(Status::Paused));
    assert_eq!(
        submit(&mut runtime, lease, Action::Next, 300).result,
        Err(Code::State)
    );
    fixture.policy.0.borrow_mut().deny = None;
    apply(&mut runtime, lease, Action::Resume, 300);
    runtime.tick(400).unwrap();
    assert_eq!(runtime.state().elapsed_ms, 200);
    apply(&mut runtime, lease, Action::Next, 400);
    assert_eq!(runtime.state().instance, instance);
    assert_eq!(runtime.state().step, Some(runtime.steps()[1].id));
    {
        let policy = fixture.policy.0.borrow();
        let permissions = &policy.permissions;
        assert_eq!(permissions.last().unwrap().action, PermissionAction::Next);
        assert_eq!(permissions.last().unwrap().instance, instance.unwrap());
    }
    fixture.policy.0.borrow_mut().deny = Some(Denial::Missing);
    apply(&mut runtime, lease, Action::Stop, 400);
    assert!(runtime.state().instance.is_none());
}
#[test]
fn clocks_and_stale_revisions_do_not_apply_delayed_commands() {
    let fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Remote, 0);
    load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 100);
    let stale = request(&runtime, lease, Action::Stop);
    let before = runtime.state();
    assert_eq!(runtime.submit(stale, 99), Err(Code::Clock));
    assert_eq!(runtime.state(), before);
    assert_eq!(
        runtime.acquire(grant(Origin::Panel, 1000), true, 99),
        Err(Code::Clock)
    );
    assert_eq!(runtime.state(), before);
    apply(&mut runtime, lease, Action::Pause, 100);
    let newer = Request {
        serial: runtime.state().owner.unwrap().serial + 1,
        ..stale
    };
    let receipt = runtime.submit(newer, 101).unwrap();
    assert_eq!(receipt.result, Err(Code::Revision));
    assert_eq!(runtime.state().status, Some(Status::Paused));
    let repeated = runtime.submit(newer, 102).unwrap();
    assert_eq!(repeated, receipt);
    let gap = Request {
        serial: newer.serial + 2,
        ..newer
    };
    assert_eq!(runtime.submit(gap, 102), Err(Code::Sequence));
}
#[test]
fn maintenance_requires_current_output_ack_and_releases_all_readers_before_installing() {
    let mut fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 0);
    assert_eq!(
        submit(&mut runtime, lease, Action::BeginMaintenance, 0).result,
        Err(Code::Busy)
    );
    apply(&mut runtime, lease, Action::Stop, 10);
    apply(&mut runtime, lease, Action::BeginMaintenance, 10);
    let stale = runtime.quiescence_request().unwrap();
    assert_eq!(fixture.metrics.readers.get(), 1);
    assert!(runtime.render(&mut [0; 512]).unwrap().is_none());
    apply(&mut runtime, lease, Action::CancelMaintenance, 10);
    assert_eq!(runtime.confirm_quiescent(stale, 10), Err(Code::Mode));
    assert_eq!(fixture.metrics.readers.get(), 1);
    apply(&mut runtime, lease, Action::BeginMaintenance, 10);
    let ticket = runtime.quiescence_request().unwrap();
    assert_ne!(ticket, stale);
    let permit = runtime.confirm_quiescent(ticket, 10).unwrap();
    assert_eq!(fixture.metrics.readers.get(), 0);
    assert!(runtime.catalog().is_empty());
    assert_eq!(
        submit(&mut runtime, lease, Action::Load, 10).result,
        Err(Code::Mode)
    );
    assert_eq!(
        submit(&mut runtime, lease, Action::CancelMaintenance, 10).result,
        Err(Code::Mode)
    );
    let replacement = package(false);
    runtime
        .with_maintenance(permit, || {
            install(&mut fixture.installer, &replacement, 2);
            install(&mut fixture.installer, &fixture.bytes, 3); // Reuse formerly leased slot A.
            install(&mut fixture.installer, &replacement, 4);
        })
        .unwrap();
    fixture.metrics.fail.set(true);
    let failed = runtime.finish_maintenance(permit, 10, || fixture.installer.snapshot().map(Some));
    assert!(matches!(
        failed,
        Err(MaintenanceError::Load(stagemaster_install::Error::Package(
            stagemaster_package::Error::Read
        )))
    ));
    assert_eq!(fixture.metrics.readers.get(), 0);
    fixture.metrics.fail.set(false);
    assert_eq!(runtime.state().mode, Mode::Maintenance);
    runtime
        .finish_maintenance(permit, 10, || fixture.installer.snapshot().map(Some))
        .unwrap();
    let called = Cell::new(false);
    assert_eq!(
        runtime.with_maintenance(permit, || called.set(true)),
        Err(Code::Mode)
    );
    assert!(!called.get());
    assert!(runtime.state().selected.is_none());
    assert!(runtime.state().loaded.is_none());
    assert!(runtime.state().instance.is_none());
    assert_eq!(
        runtime.state().bound_package.unwrap(),
        fixture.installer.head().unwrap()
    );
}
#[test]
fn initial_output_handshake_and_empty_recovery_do_not_start_a_program() {
    let fixture = Fixture::new(false);
    let mut runtime: Device =
        Runtime::new([5; 16], 0, MAX_LOADER_BYTES, fixture.policy.clone()).unwrap();
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    assert_eq!(
        submit(&mut runtime, lease, Action::Load, 0).result,
        Err(Code::Mode)
    );
    let permit = runtime
        .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
        .unwrap();
    let other: Device = Runtime::new([9; 16], 0, MAX_LOADER_BYTES, fixture.policy.clone()).unwrap();
    assert_eq!(
        runtime.confirm_quiescent(other.quiescence_request().unwrap(), 0),
        Err(Code::Mode)
    );
    runtime
        .finish_maintenance(permit, 0, || Ok::<_, ()>(None))
        .unwrap();
    assert_eq!(
        submit(&mut runtime, lease, Action::Load, 0).result,
        Err(Code::Empty)
    );
    assert!(runtime.render(&mut [0; 512]).unwrap().is_none());
}

#[test]
fn denied_restart_and_unknown_steps_preserve_the_current_instance() {
    let fixture = Fixture::new(true);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 0);
    let original = runtime.state().instance;
    assert_eq!(
        submit(&mut runtime, lease, Action::Start { step: [255; 16] }, 100).result,
        Err(Code::Step)
    );
    assert_eq!(runtime.state().instance, original);
    fixture.policy.0.borrow_mut().deny = Some(Denial::Restricted);
    let step = runtime.steps()[0].id;
    assert_eq!(
        submit(&mut runtime, lease, Action::Start { step }, 100).result,
        Err(Code::Permission(Denial::Restricted))
    );
    assert_eq!(runtime.state().instance, original);
    assert_eq!(runtime.state().elapsed_ms, 100);
    let permission = *fixture.policy.0.borrow().permissions.last().unwrap();
    assert_eq!(
        permission.package,
        runtime.state().bound_package.unwrap().identity
    );
    assert_eq!(permission.program, runtime.state().loaded.unwrap());
    assert_eq!(permission.action, PermissionAction::Start);
    assert_ne!(permission.instance, original.unwrap());
    fixture.policy.0.borrow_mut().deny = None;
    start(&mut runtime, lease, 100);
    assert_eq!(runtime.state().instance, Some(permission.instance));
    assert_eq!(runtime.state().elapsed_ms, 0);
}

#[test]
fn finished_programs_keep_their_last_frame_until_explicit_stop() {
    let fixture = Fixture::new(false);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Panel, 0);
    load(&mut runtime, lease, 2, 0);
    start(&mut runtime, lease, 0);
    runtime.tick(8000).unwrap();
    assert_eq!(runtime.state().status, Some(Status::Finished));
    assert!(runtime.state().instance.is_some());
    for action in [
        Action::Load,
        Action::BeginMaintenance,
        Action::Resume,
        Action::Pause,
        Action::Next,
    ] {
        assert!(submit(&mut runtime, lease, action, 8000).result.is_err());
    }
    let mut last = [0; 512];
    runtime.render(&mut last).unwrap().unwrap();
    runtime.tick(9000).unwrap();
    let mut held = [0; 512];
    runtime.render(&mut held).unwrap().unwrap();
    assert_eq!(last, held);
    apply(&mut runtime, lease, Action::Stop, 9000);
    apply(&mut runtime, lease, Action::BeginMaintenance, 9000);
    assert_eq!(runtime.state().mode, Mode::Quiescing);
}

#[test]
fn bad_grants_and_wrong_lease_requests_cannot_revoke_a_valid_owner() {
    let fixture = Fixture::new(false);
    let mut runtime = fixture.runtime(MAX_LOADER_BYTES);
    let lease = acquire(&mut runtime, Origin::Remote, 0);
    for bad in [
        Grant {
            principal: [0; 16],
            ..grant(Origin::Panel, 10)
        },
        grant(Origin::Panel, 0),
        grant(Origin::Panel, 60001),
    ] {
        assert_eq!(runtime.acquire(bad, true, 0), Err(Code::Identity));
        assert_eq!(runtime.state().owner.unwrap().lease, lease);
    }
    assert_eq!(runtime.renew(lease, 60001, 0), Err(Code::Identity));
    runtime.renew(lease, 1000, 1).unwrap();
    let forged = Request {
        lease: Lease {
            boot: [3; 16],
            ..lease
        },
        ..request(&runtime, lease, Action::Stop)
    };
    assert_eq!(runtime.submit(forged, 10), Err(Code::Lease));
    assert_eq!(runtime.state().owner.unwrap().lease, lease);
    runtime.tick(1000).unwrap();
    assert!(runtime.state().owner.is_some());
    runtime.tick(1001).unwrap();
    assert!(runtime.state().owner.is_none());
    assert!(
        Runtime::<Reader, PolicyRef>::new([0; 16], 0, MAX_LOADER_BYTES, fixture.policy.clone())
            .is_err()
    );
    assert!(
        Runtime::<Reader, PolicyRef>::new([1; 16], 0, MAX_LOADER_BYTES + 1, fixture.policy.clone())
            .is_err()
    );
}

#[test]
fn stale_maintenance_handles_never_call_work_or_snapshot_factories() {
    let fixture = Fixture::new(false);
    let mut runtime: Device =
        Runtime::new([3; 16], 0, MAX_LOADER_BYTES, fixture.policy.clone()).unwrap();
    let permit = runtime
        .confirm_quiescent(runtime.quiescence_request().unwrap(), 0)
        .unwrap();
    let called = Cell::new(false);
    assert_eq!(
        runtime.finish_maintenance(permit, 0, || Err::<
            Option<stagemaster_install::Installed<Reader>>,
            _,
        >("unavailable")),
        Err(MaintenanceError::Load("unavailable"))
    );
    runtime
        .finish_maintenance(permit, 0, || Ok::<_, ()>(None))
        .unwrap();
    assert_eq!(
        runtime.finish_maintenance(permit, 0, || {
            called.set(true);
            Ok::<_, ()>(None)
        }),
        Err(MaintenanceError::State(Code::Mode))
    );
    assert!(!called.get());
    let lease = acquire(&mut runtime, Origin::Panel, 1);
    apply(&mut runtime, lease, Action::BeginMaintenance, 1);
    let fresh = runtime
        .confirm_quiescent(runtime.quiescence_request().unwrap(), 1)
        .unwrap();
    assert_eq!(
        runtime.with_maintenance(permit, || called.set(true)),
        Err(Code::Mode)
    );
    assert!(!called.get());
    assert_eq!(
        runtime.finish_maintenance(fresh, 0, || {
            called.set(true);
            Ok::<_, ()>(None)
        }),
        Err(MaintenanceError::State(Code::Clock))
    );
    assert!(!called.get());
    runtime
        .finish_maintenance(fresh, 1, || fixture.installer.snapshot().map(Some))
        .unwrap();
    assert_eq!(runtime.state().mode, Mode::Operation);
    assert!(runtime.state().selected.is_none());
}
