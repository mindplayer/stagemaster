#![cfg(feature = "application")]
#[allow(dead_code)]
mod maintenance_support;
#[allow(dead_code)]
mod operation_support;
#[allow(dead_code)]
mod runtime_queue_support;
use maintenance_support::{fixture, install};
use runtime_queue_support::Link;
use stagemaster_install_worker::runtime_queue::{Endpoint, Error};
use stagemaster_runtime::{Action, ProgramKey};
use stagemaster_runtime_protocol::Operation;
use std::{cell::Cell, rc::Rc};

#[test]
fn cancellation_between_queue_and_execution_does_not_acquire_input() {
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
    link.close();
    let completion = endpoint.process(&mut device, command, || 1, || link.slot.get());
    assert_eq!(device.state().owner, None);
    assert_eq!(link.complete(completion, 1), Err(Error::Closed));
}

#[test]
fn live_slot_is_reread_after_actual_package_loading_not_cached_at_queue_time() {
    let (_dir, mut device, metrics, bytes) = fixture();
    install(&mut device, &bytes);
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    );
    link.ok(&mut device, &mut endpoint, Operation::FinishMaintenance, 0);
    let item = &device.catalog()[0];
    let key = ProgramKey {
        kind: item.kind,
        id: item.id,
    };
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Apply(Action::Select(key)),
        0,
    );
    let request = link.request(&device, Operation::Apply(Action::Load));
    let command = link.queue(request, 0);
    let slot = Rc::new(Cell::new(link.slot.get()));
    let clock = Rc::new(Cell::new(0));
    let reads = Rc::new(Cell::new(0));
    let revoked = slot.clone();
    let delayed = clock.clone();
    let count = reads.clone();
    *metrics.during_read.borrow_mut() = Some(Box::new(move || {
        count.set(count.get() + 1);
        revoked.set(None);
        delayed.set(500);
    }));
    let completion = endpoint.process(&mut device, command, || clock.get(), || slot.get());
    assert!(reads.get() > 0, "must reach the original package reader");
    assert_eq!(device.state().owner, None);
    assert_eq!(
        device.state().loaded,
        Some(key),
        "load may finish; revocation is not rollback"
    );
    // The radio side may not yet have observed cancellation: worker still refuses a success.
    assert_eq!(
        link.complete(completion, 500),
        Err(Error::Worker(
            stagemaster_install_worker::operations::Error::Obsolete
        ))
    );
    assert!(link.slot.get().is_none());
}

#[test]
fn old_queue_and_completion_cannot_evict_a_new_connection_or_its_owner() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut old = Link::open(&mut device, &mut endpoint, 1, 0);
    let request = old.request(
        &device,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
    );
    let stale = old.queue(request, 0);
    old.close();
    let mut new = Link::open(&mut device, &mut endpoint, 2, 0);
    new.ok(
        &mut device,
        &mut endpoint,
        Operation::Acquire {
            duration_ms: 1000,
            takeover: false,
        },
        0,
    );
    let owner = device.state().owner;
    let request = new.request(&device, Operation::Status);
    let current = new.queue(request, 0);
    let obsolete = endpoint.process(&mut device, stale, || 0, || new.slot.get());
    assert_eq!(new.complete(obsolete, 0), Ok(false));
    assert_eq!(device.state().owner, owner);
    assert!(new.work(&mut device, &mut endpoint, current, 0).unwrap());
    new.outgoing(0);
    assert_eq!(device.state().owner, owner);
}

#[test]
fn expired_published_receive_lease_releases_input_without_waiting_for_radio_poll() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Acquire {
            duration_ms: 10_000,
            takeover: false,
        },
        0,
    );
    let current = link.slot.get().unwrap();
    assert!(current.grant(5999).is_some());
    assert!(current.grant(6000).is_none());
    endpoint
        .tick(&mut device, 6000, || link.slot.get())
        .unwrap();
    assert_eq!(device.state().owner, None);
    assert!(link.gateway.live(6000).is_none());
}

#[test]
fn heartbeat_publication_after_a_worker_clock_sample_does_not_revoke_valid_input() {
    let (_dir, mut device, _metrics, _bytes) = fixture();
    let mut endpoint = Endpoint::default();
    let mut link = Link::open(&mut device, &mut endpoint, 1, 0);
    link.ok(
        &mut device,
        &mut endpoint,
        Operation::Acquire {
            duration_ms: 10_000,
            takeover: false,
        },
        0,
    );
    let owner = device.state().owner;
    // Worker samples 1000, then radio publishes a heartbeat at 1001 before it reads.
    // Both use the same monotonic clock; the fresher slot must not look like revocation.
    link.heartbeat(1001);
    endpoint
        .tick(&mut device, 1000, || link.slot.get())
        .unwrap();
    assert_eq!(device.state().owner, owner);
    assert!(link.slot.get().unwrap().grant(7000).is_some());
    assert!(link.slot.get().unwrap().grant(7001).is_none());
}
