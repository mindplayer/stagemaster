#[path = "../../../stagemaster-live/tests/support/mod.rs"]
mod fixtures;
use super::*;
use crate::{Action, LiveBackend};
use stagemaster_live::{
    Session,
    media::{GroupSpec, Limits, Preparer, Sample},
};
use stagemaster_project::Document;
use stagemaster_runtime::{Grant, Origin, Request};
use stagemaster_runtime_host::Backend;
use stagemaster_time::{Clock, Exchange, Mapping};

fn setup() -> (LiveBackend, MediaPort, Preparer, Document, Sample, Mapping) {
    setup_controlled(false)
}

pub(super) fn setup_controlled(
    controlled: bool,
) -> (LiveBackend, MediaPort, Preparer, Document, Sample, Mapping) {
    let doc = fixtures::decode(&fixtures::fixture(0));
    let clock = Clock::new([60; 16], 0).unwrap();
    let group = GroupSpec {
        id: [50; 16],
        clock,
        sources: vec![[1; 16]],
        limits: Limits {
            max_age_ns: 1_000_000_000,
            max_uncertainty_ns: 1_000_000,
            max_gap_ms: 1000,
            max_rate_percent: 100,
            position_tolerance_ms: 1,
        },
    };
    let session =
        Session::prepare_with_media(&doc, [9; 16], &fixtures::specs(), &[group], 1000).unwrap();
    let preparer = session
        .media_preparer(session.media_key([50; 16]).unwrap())
        .unwrap();
    let sample = Sample {
        progress: None,
        at: clock.at(1_000_000_000),
        sequence: 1,
        position_ms: 0,
        playing: true,
    };
    let map = Mapping::measure(
        Exchange {
            sent: sample.at,
            received: sample.at,
            received_at_target: session.host_clock().at(1_000_000_000),
            sent_at_target: session.host_clock().at(1_000_000_000),
        },
        stagemaster_time::Limits {
            max_round_trip_ns: 1,
            max_age_ns: 10_000_000_000,
            max_uncertainty_ns: 1,
            relative_drift_ppm: 0,
            timestamp_error_ns: 0,
        },
    )
    .unwrap();
    let (backend, mut ports) = if controlled {
        LiveBackend::with_controlled_media(
            session,
            &[ControlSpec {
                group: [50; 16],
                duration_ms: 1000,
                timeout_ms: 50,
            }],
        )
    } else {
        LiveBackend::with_media(session)
    }
    .unwrap();
    (backend, ports.remove(0), preparer, doc, sample, map)
}

#[test]
fn staged_plans_are_bounded_and_retained_until_caller_reclamation() {
    let (mut backend, port, preparer, doc, sample, map) = setup();
    let key = port.initial().key;
    let prepare = || preparer.prepare(key, &doc, 0, true, 2000).unwrap();
    let ticket = port.stage(prepare(), sample, map).unwrap();
    assert!(matches!(
        port.stage(prepare(), sample, map),
        Err(Code::Busy)
    ));
    assert!(matches!(port.reclaim(ticket, false), Err(Code::Busy)));
    let guard = port.staging.lock().unwrap();
    assert_eq!(backend.activate_media(ticket, 1001), Err(Code::Busy));
    drop(guard);
    assert_eq!(backend.activate_media(ticket, 1001), Ok(()));
    assert_eq!(backend.activate_media(ticket, 1001), Err(Code::State));
    assert!(matches!(
        port.stage(prepare(), sample, map),
        Err(Code::Busy)
    ));
    let reclaimed = port.reclaim(ticket, false).unwrap();
    assert_eq!(reclaimed.result, Some(Ok(())));
    assert_eq!(reclaimed.prepared.key(), key);
    drop(reclaimed); // Only this preparation-side caller drops the displaced plans.
    assert!(matches!(port.reclaim(ticket, true), Err(Code::State)));
}

#[test]
fn withdrawal_invalidates_queued_ticket_and_exhaustion_never_reuses_it() {
    let (mut backend, port, preparer, doc, sample, map) = setup();
    let key = port.initial().key;
    let prepare = || preparer.prepare(key, &doc, 0, true, 2000).unwrap();
    let ticket = port.stage(prepare(), sample, map).unwrap();
    assert_eq!(port.reclaim(ticket, true).unwrap().result, None);
    assert_eq!(backend.activate_media(ticket, 1001), Err(Code::State));
    assert_eq!(backend.state().media[0].unwrap().group.key, key);
    port.staging.lock().unwrap().serial = u64::MAX;
    assert!(matches!(
        port.stage(prepare(), sample, map),
        Err(Code::Exhausted)
    ));
    port.inbox.lock().unwrap().serial = u64::MAX;
    assert_eq!(port.publish(key, sample, map), Err(Code::Exhausted));
}

#[test]
fn rejected_authority_does_not_consume_preparation_and_busy_observation_never_blocks_tick() {
    let (mut backend, port, preparer, doc, sample, map) = setup();
    let key = port.initial().key;
    let ticket = port
        .stage(
            preparer.prepare(key, &doc, 0, true, 2000).unwrap(),
            sample,
            map,
        )
        .unwrap();
    let grant = |n| Grant {
        principal: [n; 16],
        origin: Origin::Remote,
        duration_ms: 1000,
    };
    let old = backend.acquire(grant(1), false, 1000).unwrap();
    let current = backend.acquire(grant(2), true, 1000).unwrap();
    assert_eq!(
        backend.submit(
            Request {
                lease: old,
                serial: 1,
                expected_revision: 2,
                action: Action::ActivateMedia { ticket }
            },
            1001
        ),
        Err(Code::Lease)
    );
    assert!(matches!(port.reclaim(ticket, false), Err(Code::Busy)));
    let receipt = backend
        .submit(
            Request {
                lease: current,
                serial: 1,
                expected_revision: 2,
                action: Action::ActivateMedia { ticket },
            },
            1001,
        )
        .unwrap();
    assert_eq!(receipt.result, Ok(()));
    let guard = port.inbox.lock().unwrap();
    backend.tick(1002).unwrap();
    assert_eq!(backend.state().observed_ms, 1002);
    assert!(backend.state().media[0].unwrap().observation.is_none());
    drop(guard);
}

#[test]
fn latest_observation_is_bounded_and_failed_preparations_are_reclaimed_after_shutdown() {
    let (mut backend, port, preparer, doc, sample, map) = setup();
    let key = port.initial().key;
    let expired = preparer.prepare(key, &doc, 0, true, 1001).unwrap();
    let ticket = port.stage(expired, sample, map).unwrap();
    assert_eq!(backend.activate_media(ticket, 1001), Err(Code::State));
    assert_eq!(
        port.reclaim(ticket, false).unwrap().result,
        Some(Err(Code::State))
    );
    let ticket = port
        .stage(
            preparer.prepare(key, &doc, 0, true, 2000).unwrap(),
            sample,
            map,
        )
        .unwrap();
    backend.activate_media(ticket, 1001).unwrap();
    let active = backend.state().media[0].unwrap().group.key;
    for sequence in 2..=1000 {
        port.publish(
            active,
            Sample {
                sequence,
                at: sample.at.clock.at(1_002_000_000),
                ..sample
            },
            map,
        )
        .unwrap();
    }
    backend.tick(1003).unwrap();
    let receipt = backend.state().media[0].unwrap().observation.unwrap();
    assert_eq!(receipt.sample_sequence, 1000);
    assert_eq!(receipt.generation, active.generation());
    assert_eq!(receipt.result, Ok(()));
    port.publish(key, sample, map).unwrap();
    backend.tick(1004).unwrap();
    let state = backend.state().media[0].unwrap();
    assert_eq!(state.group.key, active);
    assert_eq!(state.observation.unwrap().generation, key.generation());
    assert_eq!(state.observation.unwrap().result, Err(Code::State));
    drop(backend);
    assert_eq!(port.publish(active, sample, map), Err(Code::State));
    assert_eq!(port.reclaim(ticket, false).unwrap().result, Some(Ok(())));
    assert!(matches!(
        port.stage(
            preparer.prepare(key, &doc, 0, true, 2000).unwrap(),
            sample,
            map
        ),
        Err(Code::State)
    ));
}
