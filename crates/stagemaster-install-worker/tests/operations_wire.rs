#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
mod operation_wire_support;
use maintenance_support::{fixture, install};
use operation_support::{access, all};
use operation_wire_support::Link;
use stagemaster_device_auth::application::{Permissions, Scope};
use stagemaster_install_worker::operations::{Connection, Error};
use stagemaster_runtime::{Action, Code, Status};
use stagemaster_runtime_protocol::{Body, Failure, Offer, Operation, Request};

#[test]
fn encrypted_negotiation_directory_and_playback_reuse_the_original_runtime() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut link = Link::open(&mut device, all(), 0);
    link.ok(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    );
    link.ok(&mut device, Operation::FinishMaintenance, 0);
    let index = u16::try_from(
        device
            .catalog()
            .iter()
            .position(|p| p.kind == stagemaster_package::Kind::Sequence)
            .unwrap(),
    )
    .unwrap();
    let catalog = link.ok(&mut device, Operation::Catalog { index }, 0);
    let Body::Program(Some(program)) = catalog.body else {
        panic!("missing installed program")
    };
    link.ok(
        &mut device,
        Operation::Apply(Action::Select(program.key)),
        0,
    );
    link.ok(&mut device, Operation::Apply(Action::Load), 0);
    let page = link.ok(&mut device, Operation::Step { index: 0 }, 0);
    let Body::Step(Some(step)) = page.body else {
        panic!("missing loaded step")
    };
    assert_eq!(step.name.as_str(), device.steps()[0].name);
    let started = link.ok(
        &mut device,
        Operation::Apply(Action::Start { step: step.id }),
        0,
    );
    let instance = device.state().instance;
    // Same plaintext request, new encrypted record nonce; a lost reply does not re-execute.
    assert_eq!(
        link.request(&mut device, started.request, 100).unwrap(),
        started
    );
    assert_eq!(device.state().instance, instance);
    let loaded = device.snapshot().unwrap().load(usize::from(index)).unwrap();
    let mut reference = stagemaster_playback::Player::new(loaded.plan, 0);
    reference.execute(0, 0).unwrap();
    reference.advance(100).unwrap();
    let mut expected = [0; 512];
    let mut actual = [0; 512];
    loaded
        .output
        .render(reference.values(), &mut expected)
        .unwrap();
    device.render(&mut actual).unwrap().unwrap();
    assert_eq!(actual, expected);
    link.ok(&mut device, Operation::Apply(Action::Pause), 100);
    device.tick(200).unwrap();
    link.ok(&mut device, Operation::Apply(Action::Resume), 200);
    link.ok(&mut device, Operation::Apply(Action::Next), 250);
    link.connection.close(&mut device).unwrap();
    link.access.revoke();
    link.client.close();
    device.tick(300).unwrap();
    assert_eq!(device.state().instance, instance);
    assert_eq!(device.state().status, Some(Status::Running));
    assert_eq!(device.state().owner, None);
    let mut reconnect = Link::open(&mut device, all(), 300);
    let status = reconnect.ok(&mut device, Operation::Status, 300);
    assert!(matches!(status.body, Body::State { state, .. } if state.instance == instance));
    reconnect.ok(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        300,
    );
    reconnect.ok(&mut device, Operation::Apply(Action::Stop), 300);
    assert_eq!(device.state().instance, None);
}

#[test]
fn encrypted_observer_and_stale_revision_cannot_change_control_or_program() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut owner = Link::open(&mut device, all(), 0);
    owner.ok(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    );
    let state = device.state();
    let mut observer = Link::open(&mut device, Permissions::only(Scope::Observe), 0);
    observer.ok(&mut device, Operation::Status, 0);
    assert_eq!(
        observer.send(&mut device, Operation::Apply(Action::Stop), 0),
        Err(Error::Denied)
    );
    assert_eq!(device.state(), state);
    let outdated = Request {
        session: owner.ready.peer.session,
        id: 2,
        expected_revision: state.revision + 1,
        operation: Operation::Release,
    };
    let reply = owner.request(&mut device, outdated, 0).unwrap();
    assert!(matches!(
        reply.body,
        Body::State {
            result: Err(Failure::Runtime(Code::Revision)),
            ..
        }
    ));
    assert_eq!(device.state(), state);
    assert_eq!(owner.request(&mut device, outdated, 0).unwrap(), reply);
}

#[test]
fn malformed_or_unnegotiated_messages_fail_closed_without_maintenance() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let initial = device.state();
    let mut session = access(initial.boot, 9, all(), 0);
    let mut connection = Connection::open(&device, 0, |t| session.grant(t).ok()).unwrap();
    assert_eq!(
        connection.process_message(&mut device, &[], || 0, |t| session.grant(t).ok()),
        Err(Error::Negotiation)
    );
    assert_eq!(device.state(), initial);
    let mut link = Link::open(&mut device, all(), 0);
    link.ok(
        &mut device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    );
    assert!(matches!(
        link.raw(&mut device, b"SMAP\x01\x01\x70\0", 0),
        Err(Error::Protocol(_))
    ));
    assert_eq!(device.state().owner, None);
    assert_eq!(device.state().mode, initial.mode);
    assert!(
        link.connection
            .poll(&mut device, 0, |t| link.access.grant(t).ok())
            .is_err()
    );
    let mut another = Link::open(&mut device, all(), 0);
    assert_eq!(
        another.connection.negotiate(
            &mut device,
            Offer::current().encode().unwrap().bytes(),
            0,
            |t| another.access.grant(t).ok()
        ),
        Err(Error::Negotiation)
    );
}
