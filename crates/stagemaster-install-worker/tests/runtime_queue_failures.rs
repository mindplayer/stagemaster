#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
#[allow(dead_code)]
mod runtime_queue_support;
use maintenance_support::{epoch, fixture};
use operation_support::{access, all};
use runtime_queue_support::Link;
use stagemaster_device_auth::application::{Permissions, Scope};
use stagemaster_device_session::Kind;
use stagemaster_install_worker::runtime_queue::{Command, Completion, Endpoint, Error, Gateway};
use stagemaster_runtime_protocol::{Operation, Response};

#[test]
fn installation_credentials_and_observer_control_cannot_open_control_authority() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let install = access(
        device.state().boot,
        9,
        Permissions::only(Scope::Installation),
        0,
    );
    assert!(Gateway::new(install, epoch(1), 0).is_err());
    let mut endpoint = Endpoint::default();
    let mut link = Link::new(&device, 1, Permissions::only(Scope::Observe), 0);
    link.negotiate(&mut device, &mut endpoint, 0);
    link.ok(&mut device, &mut endpoint, Operation::Status, 0);
    let request = link.request(
        &device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    );
    let command = link.queue(request, 0);
    assert_eq!(
        link.work(&mut device, &mut endpoint, command, 0),
        Err(Error::Worker(
            stagemaster_install_worker::operations::Error::Denied
        ))
    );
    assert_eq!(device.state().owner, None);
    assert!(link.slot.get().is_none());
}

#[test]
fn changed_pending_request_and_wrong_session_fail_closed_without_work() {
    for foreign in [false, true] {
        let (_dir, mut device, _metrics, _bytes) = fixture();
        let mut endpoint = Endpoint::default();
        let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
        let mut request = link.request(&device, Operation::Status);
        let original = link.queue(request, 0);
        if foreign {
            request.session = [33; 16];
        } else {
            request.operation = Operation::Release;
        }
        assert!(
            link.receive(Kind::Message, request.encode().unwrap().bytes(), 0)
                .is_err()
        );
        let completion = endpoint.process(&mut device, original, || 0, || link.slot.get());
        assert_eq!(link.complete(completion, 0), Err(Error::Closed));
        assert_eq!(device.state().owner, None);
    }
}

#[test]
fn current_ciphertext_stays_immutable_when_work_completes_during_a_heartbeat_send() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    let request = link.request(&device, Operation::Status);
    let command = link.queue(request, 0);
    assert!(link.receive(Kind::Heartbeat, &[], 100).unwrap().is_none());
    let in_flight = link.gateway.outbound(100).unwrap().unwrap().to_vec();
    link.work(&mut device, &mut endpoint, command, 100).unwrap();
    assert_eq!(link.gateway.outbound(200).unwrap().unwrap(), in_flight);
    assert_eq!(link.outgoing(200), (Kind::HeartbeatReply, Vec::new()));
    let (kind, bytes) = link.outgoing(200);
    assert_eq!(kind, Kind::Message);
    let response = Response::decode(&bytes).unwrap();
    response.correlate(request, device.state().boot).unwrap();
}

#[test]
fn opening_and_reply_deadlines_do_not_slide_when_authenticated_traffic_arrives() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut link = Link::new(&device, 1, all(), 0);
    link.heartbeat(2000);
    link.heartbeat(4000);
    assert_eq!(link.gateway.poll(5000), Err(Error::Expired));
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 2, 0);
    let request = link.request(&device, Operation::Status);
    let command = link.queue(request, 0);
    link.work(&mut device, &mut endpoint, command, 0).unwrap();
    let cipher = link.gateway.outbound(0).unwrap().unwrap().to_vec();
    // New incoming traffic cannot replace an in-flight reply or move its deadline.
    link.receive(Kind::Heartbeat, &[], 4000).unwrap();
    assert_eq!(link.gateway.outbound(4999).unwrap().unwrap(), cipher);
    assert_eq!(link.gateway.poll(5000), Err(Error::Expired));
}

#[test]
fn live_snapshot_rejects_future_time_and_backwards_poll_closes() {
    let (_dir, device, _metrics, _bytes) = fixture();
    let mut link = Link::new(&device, 1, all(), 100);
    let current = link.slot.get().unwrap();
    assert!(current.grant(99).is_none());
    assert!(current.grant(100).is_some());
    assert_eq!(link.gateway.poll(99), Err(Error::Clock));
    assert!(link.gateway.live(100).is_none());
}

#[test]
fn queue_memory_is_fixed_and_has_no_frame_copy_in_the_command() {
    assert!(size_of::<Command>() <= 256);
    assert!(size_of::<Completion>() <= 1536);
    assert!(size_of::<Gateway>() <= 4096);
    assert!(size_of::<Endpoint>() <= 2048);
}

#[test]
fn late_same_connection_completion_cannot_finish_the_next_request() {
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
    let original = endpoint.process(&mut device, command, || 0, || link.slot.get());
    let owner = device.state().owner;
    let delayed = endpoint.process(&mut device, command, || 0, || link.slot.get());
    assert_eq!(
        device.state().owner,
        owner,
        "duplicate work uses original history"
    );
    link.complete(original, 0).unwrap();
    link.outgoing(0);
    let next = link.request(&device, Operation::Status);
    let next_command = link.queue(next, 0);
    assert_eq!(link.complete(delayed, 0), Ok(false));
    assert!(link.gateway.outbound(0).unwrap().is_none());
    link.work(&mut device, &mut endpoint, next_command, 0)
        .unwrap();
    let (_, bytes) = link.outgoing(0);
    Response::decode(&bytes)
        .unwrap()
        .correlate(next, device.state().boot)
        .unwrap();
}

#[test]
fn delayed_negotiation_receipt_reduces_remaining_permission_before_delivery() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::new(&device, 1, all(), 0);
    let command = link.offer(0);
    let completion = endpoint.process(&mut device, command, || 0, || link.slot.get());
    link.complete(completion, 4000).unwrap();
    let (_, bytes) = link.outgoing(4000);
    let ready = stagemaster_runtime_protocol::Ready::decode(&bytes).unwrap();
    assert_eq!(ready.remaining_ms, 56_000);
}
