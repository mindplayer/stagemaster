#![cfg(feature = "application")]
mod firmware_application_support;
#[allow(dead_code)]
mod maintenance_support;
use firmware_application_support::Link;
use maintenance_support::{fixture, install};
use stagemaster_device_link::management::ApplicationReceipt;
use stagemaster_device_session::Kind;
use stagemaster_runtime::{Action, Status};
use stagemaster_runtime_protocol::{Body, Operation};

#[test]
fn actual_firmware_handshake_routes_legacy_installation_and_preserves_transfer() {
    for (version, bits) in [(1, 0), (2, 7)] {
        let (_dir, mut device, _metrics, bytes) = fixture();
        device
            .confirm_quiescent(device.quiescence_request().unwrap(), 0)
            .unwrap();
        let mut link = Link::connect(&device, false, version, bits).unwrap();
        assert!(link.slots.installation().is_some());
        assert!(link.slots.runtime().is_none());
        link.pump(&mut device, 0).unwrap();
        let (_, receipt) = link.outgoing(0);
        let receipt = ApplicationReceipt::decode(&receipt).unwrap();
        let mut upload = stagemaster_transfer::Upload::new(bytes.as_slice()).unwrap();
        upload.connect(receipt.session).unwrap();
        while let Some(frame) = upload.outbound().unwrap().cloned() {
            link.receive(Kind::Message, frame.bytes(), 0).unwrap();
            link.pump(&mut device, 0).unwrap();
            upload.accept(&link.outgoing(0).1).unwrap();
        }
        assert!(device.snapshot().unwrap().load(0).is_ok());
        let slots = link.slots.clone();
        drop(link);
        assert!(slots.revoked());
    }
}

#[test]
fn actual_firmware_runtime_does_not_enter_maintenance_or_grant_control_on_connect() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    device.finish_maintenance(0).unwrap();
    let initial = device.state();
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    assert!(link.slots.installation().is_none());
    link.negotiate(&mut device);
    assert_eq!(device.state(), initial);
    let catalog = link.call(&mut device, Operation::Catalog { index: 0 }, 0);
    let Body::Program(Some(program)) = catalog.body else {
        panic!("missing program")
    };
    link.call(
        &mut device,
        Operation::Acquire {
            duration_ms: 10_000,
            takeover: false,
        },
        0,
    );
    link.call(
        &mut device,
        Operation::Apply(Action::Select(program.key)),
        0,
    );
    link.call(&mut device, Operation::Apply(Action::Load), 0);
    let Body::Step(Some(step)) = link.call(&mut device, Operation::Step { index: 0 }, 0).body
    else {
        panic!("missing step")
    };
    link.call(
        &mut device,
        Operation::Apply(Action::Start { step: step.id }),
        0,
    );
    let instance = device.state().instance;
    assert!(instance.is_some());
    link.close();
    link.endpoint
        .tick(&mut device, 200, || link.slots.runtime())
        .unwrap();
    assert_eq!(device.state().owner, None);
    assert_eq!(device.state().instance, instance);
    assert_eq!(device.state().status, Some(Status::Running));
    assert!(device.state().elapsed_ms >= 200);
}

#[test]
fn runtime_refuses_legacy_or_unobserved_scope_before_publishing_authority() {
    let (_dir, device, _metrics, _bytes) = fixture();
    assert!(Link::connect(&device, true, 1, 0).is_err());
    assert!(Link::connect(&device, true, 2, 1).is_err());
    assert!(Link::connect(&device, true, 2, 4).is_err());
    assert!(Link::connect(&device, false, 2, 6).is_err());
}

#[test]
fn enqueue_failure_and_idle_expiry_clear_the_independent_firmware_slot() {
    let (_dir, device, _metrics, _bytes) = fixture();
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    link.slots.full.set(true);
    assert!(
        link.receive(
            Kind::Message,
            stagemaster_runtime_protocol::Offer::current()
                .encode()
                .unwrap()
                .bytes(),
            0
        )
        .is_err()
    );
    assert!(link.slots.revoked());
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    assert!(link.poll(5000).is_err());
    assert!(link.slots.revoked());
}

#[test]
fn firmware_protocol_keeps_heartbeats_live_while_original_work_is_queued() {
    let (_dir, mut device, _metrics, bytes) = fixture();
    install(&mut device, &bytes);
    device.finish_maintenance(0).unwrap();
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    link.negotiate(&mut device);
    let request = stagemaster_runtime_protocol::Request {
        session: link.client.peer(0).unwrap().unwrap().session(),
        id: 1,
        expected_revision: device.state().revision,
        operation: Operation::Status,
    };
    link.receive(Kind::Message, request.encode().unwrap().bytes(), 0)
        .unwrap();
    for now in [2000, 4000, 6000, 8000] {
        link.receive(Kind::Heartbeat, &[], now).unwrap();
        assert_eq!(link.outgoing(now), (Kind::HeartbeatReply, Vec::new()));
    }
    link.pump(&mut device, 8000).unwrap();
    let response = stagemaster_runtime_protocol::Response::decode(&link.outgoing(8000).1).unwrap();
    response.correlate(request, device.state().boot).unwrap();
    let slots = link.slots.clone();
    drop(link);
    assert!(slots.revoked());
}

#[test]
fn malformed_runtime_messages_and_backward_time_revoke_publication() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    link.negotiate(&mut device);
    assert!(link.receive(Kind::Message, b"invalid", 100).is_err());
    assert!(link.slots.revoked());
    let mut link = Link::connect(&device, true, 2, 7).unwrap();
    link.poll(100).unwrap();
    assert!(link.poll(99).is_err());
    assert!(link.slots.revoked());
}

#[test]
fn observation_only_firmware_configuration_cannot_acquire_control() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut link = Link::connect(&device, true, 2, 2).unwrap();
    link.negotiate(&mut device);
    link.call(&mut device, Operation::Status, 0);
    let request = stagemaster_runtime_protocol::Request {
        session: link.client.peer(0).unwrap().unwrap().session(),
        id: 2,
        expected_revision: device.state().revision,
        operation: Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    };
    link.receive(Kind::Message, request.encode().unwrap().bytes(), 0)
        .unwrap();
    assert!(link.pump(&mut device, 0).is_err());
    assert!(link.slots.revoked());
    assert!(device.state().owner.is_none());
}
