#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
#[allow(dead_code)]
mod runtime_queue_support;
use maintenance_support::{fixture, install};
use runtime_queue_support::{Link, assert_reference};
use stagemaster_device_session::Kind;
use stagemaster_install_worker::runtime_queue::Endpoint;
use stagemaster_runtime::{Action, Status};
use stagemaster_runtime_protocol::{Body, Operation};

#[test]
fn queued_real_package_playback_survives_disconnect_and_reuses_original_frames() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    link.acquire(&mut device, &mut endpoint, 0);
    link.ok(&mut device, &mut endpoint, Operation::FinishMaintenance, 0);
    let index = u16::try_from(
        device
            .catalog()
            .iter()
            .position(|p| p.kind == stagemaster_package::Kind::Sequence)
            .unwrap(),
    )
    .unwrap();
    let catalog = link.ok(&mut device, &mut endpoint, Operation::Catalog { index }, 0);
    let Body::Program(Some(program)) = catalog.body else {
        panic!("missing program")
    };
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Select(program.key)),
        0,
    );
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Load),
        0,
    );
    let page = link.ok(&mut device, &mut endpoint, Operation::Step { index: 0 }, 0);
    let Body::Step(Some(step)) = page.body else {
        panic!("missing step")
    };
    assert_eq!(step.name.as_str(), device.steps()[0].name);
    let start = link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Start { step: step.id }),
        0,
    );
    let instance = device.state().instance;
    assert_eq!(
        link.send(&mut device, &mut endpoint, start.request, 100),
        start
    );
    assert_eq!(device.state().instance, instance);
    assert_reference(&device, usize::from(index), 100);
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Pause),
        100,
    );
    endpoint.tick(&mut device, 200, || link.slot.get()).unwrap();
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Resume),
        200,
    );
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Next),
        250,
    );
    link.close();
    endpoint.tick(&mut device, 300, || link.slot.get()).unwrap();
    assert_eq!(device.state().instance, instance);
    assert_eq!(device.state().owner, None);
    assert_eq!(device.state().status, Some(Status::Running));
    let elapsed = device.state().elapsed_ms;
    endpoint.tick(&mut device, 350, || None).unwrap();
    assert!(device.state().elapsed_ms > elapsed);
    let mut new = Link::open(&mut device, &mut endpoint, 2, 350);
    new.ok(&mut device, &mut endpoint, Operation::Status, 350);
    assert_eq!(device.state().owner, None);
    new.acquire(&mut device, &mut endpoint, 350);
    new.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Stop),
        350,
    );
    assert_eq!(device.state().instance, None);
}

#[test]
fn pending_work_keeps_heartbeats_live_and_exact_retries_do_not_queue_again() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    let request = link.request(
        &device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    );
    let command = link.queue(request, 0);
    for now in [2000, 4000, 6000, 8000] {
        assert!(
            link.receive(Kind::Message, request.encode().unwrap().bytes(), now)
                .unwrap()
                .is_none()
        );
        link.heartbeat(now);
        assert!(link.slot.get().unwrap().grant(now).is_some());
    }
    assert_eq!(device.state().owner, None);
    link.work(&mut device, &mut endpoint, command, 8000)
        .unwrap();
    let owner = device.state().owner;
    // Completion already prepared; duplicates still cannot produce more work.
    assert!(
        link.receive(Kind::Message, request.encode().unwrap().bytes(), 8000)
            .unwrap()
            .is_none()
    );
    let (_, bytes) = link.outgoing(8000);
    let response = stagemaster_runtime_protocol::Response::decode(&bytes).unwrap();
    assert_eq!(
        link.send(&mut device, &mut endpoint, request, 8000),
        response
    );
    assert_eq!(device.state().owner, owner);
}

#[test]
fn heartbeats_and_retries_do_not_renew_the_thirty_second_work_deadline() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    let request = link.request(
        &device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    );
    let command = link.queue(request, 0);
    for now in (2000..30_000).step_by(2000) {
        link.heartbeat(now);
        assert!(
            link.receive(Kind::Message, request.encode().unwrap().bytes(), now)
                .unwrap()
                .is_none()
        );
    }
    // No radio poll at the cutoff: the independently read slot still denies exactly then.
    let published = link.slot.get().unwrap();
    assert!(published.grant(29_999).is_some());
    assert!(published.grant(30_000).is_none());
    let completion = endpoint.process(&mut device, command, || 30_000, || link.slot.get());
    assert_eq!(device.state().owner, None);
    assert!(link.complete(completion, 30_000).is_err());
    assert!(link.slot.get().is_none());
}
