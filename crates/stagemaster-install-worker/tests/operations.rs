#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
mod operation_support;
use maintenance_support::{fixture, install};
use operation_support::{Peer, access, all};
use stagemaster_device_auth::application::{Permissions, Scope};
use stagemaster_install_worker::operations::{Connection, Detail, Error, Failure, Operation};
use stagemaster_runtime::{Action, Code, Mode, Status};

#[test]
fn opening_and_observing_do_not_enter_maintenance_acquire_or_start() {
    let (_dir, mut device, metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let state = device.state();
    let mut installation = access(state.boot, 9, Permissions::only(Scope::Installation), 0);
    assert!(matches!(
        Connection::open(&device, 0, |t| installation.grant(t).ok()),
        Err(Error::Denied)
    ));
    let mut observer = Peer::new(&device, 10, Permissions::only(Scope::Observe), 0);
    assert_eq!(device.state(), state);
    let writes = metrics.mutations.get();
    let seen = observer.send(&mut device, Operation::Status, 0).unwrap();
    assert_eq!(seen.state.mode, Mode::Maintenance);
    assert_eq!(seen.state.owner, None);
    assert_eq!(seen.state.instance, None);
    assert_eq!(
        observer.send(
            &mut device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: true
            },
            0
        ),
        Err(Error::Denied)
    );
    assert_eq!(device.state(), state);
    assert_eq!(metrics.mutations.get(), writes);
}

#[test]
fn real_installed_program_uses_original_player_with_historical_retries_and_disconnected_playback() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut peer = Peer::new(&device, 9, all(), 0);
    let index = peer.prepare(&mut device);
    let catalog = peer
        .send(
            &mut device,
            Operation::Catalog {
                index: u16::try_from(index).unwrap(),
            },
            0,
        )
        .unwrap();
    let Detail::Program(Some(program)) = catalog.result.unwrap() else {
        panic!("missing program")
    };
    assert_eq!(program.key, device.state().loaded.unwrap());
    assert_eq!(program.name.as_str(), device.catalog()[index].name);
    let labels = peer
        .send(&mut device, Operation::Step { index: 0 }, 0)
        .unwrap();
    let Detail::Step(Some(step)) = labels.result.unwrap() else {
        panic!("missing step")
    };
    assert_eq!(step.name.as_str(), device.steps()[0].name);
    let started = peer.apply(&mut device, Action::Start { step: step.id }, 0);
    let loaded = device.snapshot().unwrap().load(index).unwrap();
    let mut reference = stagemaster_playback::Player::new(loaded.plan, 0);
    reference.execute(0, 0).unwrap();
    // Losing the start reply must not create another instance or reset its phase.
    assert_eq!(
        peer.request(&mut device, started.request, 100).unwrap(),
        started
    );
    assert_eq!(device.state().instance, started.state.instance);
    reference.advance(100).unwrap();
    let mut actual = [0; 512];
    let mut expected = [0; 512];
    device.render(&mut actual).unwrap().unwrap();
    loaded
        .output
        .render(reference.values(), &mut expected)
        .unwrap();
    assert_eq!(actual, expected);
    peer.apply(&mut device, Action::Pause, 100);
    let paused = device.state().elapsed_ms;
    device.tick(200).unwrap();
    assert_eq!(device.state().elapsed_ms, paused);
    peer.apply(&mut device, Action::Resume, 200);
    peer.apply(&mut device, Action::Next, 250);
    let instance = device.state().instance;
    peer.connection.close(&mut device).unwrap();
    device.tick(350).unwrap();
    assert_eq!(device.state().instance, instance);
    assert_eq!(device.state().status, Some(Status::Running));
    assert_eq!(device.state().owner, None);
    let mut next = Peer::new(&device, 9, all(), 350);
    next.send(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        350,
    )
    .unwrap()
    .result
    .unwrap();
    next.apply(&mut device, Action::Stop, 350);
    assert_eq!(device.state().status, Some(Status::Idle));
    assert_eq!(device.state().instance, None);
}

#[test]
fn takeover_old_disconnect_and_observation_cannot_change_the_new_owner() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut old = Peer::new(&device, 9, all(), 0);
    old.prepare(&mut device);
    let step = device.steps()[0].id;
    old.apply(&mut device, Action::Start { step }, 0);
    let mut new = Peer::new(&device, 10, all(), 10);
    let busy = new
        .send(
            &mut device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: false,
            },
            10,
        )
        .unwrap();
    assert_eq!(busy.result, Err(Failure::Runtime(Code::Busy)));
    let claimed = new
        .send(
            &mut device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: true,
            },
            10,
        )
        .unwrap();
    claimed.result.unwrap();
    let owner = device.state().owner;
    let stopped = old
        .send(&mut device, Operation::Apply(Action::Stop), 10)
        .unwrap();
    assert_eq!(stopped.result, Err(Failure::Runtime(Code::Lease)));
    old.connection.close(&mut device).unwrap();
    let mut observer = Peer::new(&device, 11, Permissions::only(Scope::Observe), 10);
    observer.send(&mut device, Operation::Status, 10).unwrap();
    observer.connection.close(&mut device).unwrap();
    assert_eq!(device.state().owner, owner);
    assert_eq!(device.state().status, Some(Status::Running));
    // Repeating acquisition returns its original expiry rather than renewing it.
    assert_eq!(
        new.request(&mut device, claimed.request, 100).unwrap(),
        claimed
    );
    assert_eq!(device.state().owner, owner);
}

#[test]
fn maintenance_requires_scope_and_actual_quiescence_then_binds_without_execution() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut peer = Peer::new(&device, 9, all(), 0);
    peer.prepare(&mut device);
    peer.apply(&mut device, Action::BeginMaintenance, 0);
    let refused = peer
        .send(&mut device, Operation::FinishMaintenance, 0)
        .unwrap();
    assert_eq!(refused.result, Err(Failure::Runtime(Code::Mode)));
    assert_eq!(device.state().mode, Mode::Quiescing);
    // This is the trusted software host's confirmation, never a remote operation.
    device
        .confirm_quiescent(device.quiescence_request().unwrap(), 0)
        .unwrap();
    let restored = peer
        .send(&mut device, Operation::FinishMaintenance, 0)
        .unwrap();
    restored.result.unwrap();
    assert_eq!(restored.state.mode, Mode::Operation);
    assert_eq!(restored.state.selected, None);
    assert_eq!(restored.state.loaded, None);
    assert_eq!(restored.state.instance, None);
    let mut controller = Peer::new(&device, 10, Permissions::only(Scope::Control), 0);
    controller
        .send(
            &mut device,
            Operation::Acquire {
                duration_ms: 1000,
                takeover: true,
            },
            0,
        )
        .unwrap()
        .result
        .unwrap();
    assert_eq!(
        controller.send(&mut device, Operation::Apply(Action::BeginMaintenance), 0),
        Err(Error::Denied)
    );
    assert_eq!(device.state().mode, Mode::Operation);
}

#[test]
fn control_permission_cannot_bypass_the_original_playback_policy() {
    struct Restricted;
    impl stagemaster_runtime::PlaybackPolicy for Restricted {
        fn authorize(
            &mut self,
            _: stagemaster_runtime::Permission,
        ) -> Result<(), stagemaster_runtime::Denial> {
            Err(stagemaster_runtime::Denial::Restricted)
        }
    }
    let (dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    drop(device);
    let store = stagemaster_install_store::FileStore::open(dir.path()).unwrap();
    let installer = stagemaster_install::Installer::open(store, [7; 16])
        .unwrap()
        .0;
    let mut device =
        stagemaster_install_worker::ManagedWorker::new(installer, 0, 64 * 1024, Restricted)
            .unwrap();
    device
        .confirm_quiescent(device.quiescence_request().unwrap(), 0)
        .unwrap();
    let mut peer = Peer::new(&device, 9, all(), 0);
    peer.prepare(&mut device);
    let step = device.steps()[0].id;
    let denied = peer
        .send(&mut device, Operation::Apply(Action::Start { step }), 0)
        .unwrap();
    assert_eq!(
        denied.result,
        Err(Failure::Runtime(Code::Permission(
            stagemaster_runtime::Denial::Restricted
        )))
    );
    assert_eq!(device.state().instance, None);
    peer.apply(&mut device, Action::Stop, 0);
}
