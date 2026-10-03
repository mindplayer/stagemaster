use super::{
    tests::{lease, submit},
    *,
};
use crate::{LiveBackend, media::tests::setup_controlled};
use stagemaster_live::media::{Prepared, Sample, Status};
use stagemaster_runtime::{Grant, Origin};
use stagemaster_runtime_host::Backend;
use stagemaster_time::{Clock, Exchange, Mapping};

fn rebound(mut sample: Sample, map: Mapping) -> (Sample, Mapping) {
    let old = sample.at.clock;
    sample.at = Clock::new(old.id(), old.epoch() + 1).unwrap().at(0);
    sample.playing = false;
    let target = map.target().at(1_000_000_000);
    let map = Mapping::measure(
        Exchange {
            sent: sample.at,
            received: sample.at,
            received_at_target: target,
            sent_at_target: target,
        },
        stagemaster_time::Limits {
            max_round_trip_ns: 1,
            max_age_ns: 1_000_000_000,
            max_uncertainty_ns: 1,
            relative_drift_ppm: 0,
            timestamp_error_ns: 0,
        },
    )
    .unwrap();
    (sample, map)
}
fn current(backend: &LiveBackend) -> ControlRequest {
    backend.state().media[0].unwrap().control.unwrap().request
}

#[test]
fn recovery_activates_paused_and_old_provider_samples_cannot_refresh_the_group() {
    let (mut backend, port, prepare, doc, old_sample, old_map) = setup_controlled(true);
    let authority = lease(&mut backend);
    let key = port.initial().key;
    let (sample, map) = rebound(old_sample, old_map);
    let plan = prepare
        .prepare(key, &doc, 0, false, 2000)
        .unwrap()
        .with_restarted_provider(sample.at.clock)
        .unwrap();
    assert_eq!(
        submit(
            &mut backend,
            authority,
            1,
            MediaCommand::Recover { position_ms: 0 },
            1000
        )
        .result,
        Ok(())
    );
    let request = current(&backend);
    let staged = port
        .stage_requested(request.ticket, plan, sample, map)
        .unwrap();
    backend.tick(1001).unwrap();
    assert_eq!(port.reclaim(staged, false).unwrap().result, Some(Ok(())));
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(1002).unwrap();
    let ready = backend.state().media[0].unwrap();
    assert_eq!(ready.group.provider, sample.at.clock);
    assert_eq!(ready.group.status, Status::Paused);
    assert_eq!(ready.control.unwrap().result, Some(Ok(())));
    assert_ne!(ready.group.key, key);
    port.publish(ready.group.key, old_sample, old_map).unwrap();
    backend.tick(1003).unwrap();
    let stale = backend.state().media[0].unwrap();
    assert_eq!(stale.observation.unwrap().result, Err(Code::State));
    assert_eq!(stale.group, ready.group);
    port.publish(
        ready.group.key,
        Sample {
            sequence: 2,
            at: sample.at.clock.at(4_000_000),
            ..sample
        },
        map,
    )
    .unwrap();
    backend.tick(1005).unwrap();
    assert_eq!(
        backend.state().media[0]
            .unwrap()
            .observation
            .unwrap()
            .result,
        Ok(())
    );
}

#[test]
fn only_explicit_recovery_can_rebind_and_completion_requires_a_real_activation() {
    let (mut backend, port, prepare, doc, original, old_map) = setup_controlled(true);
    let authority = lease(&mut backend);
    let (sample, map) = rebound(original, old_map);
    let plan = || -> Prepared {
        prepare
            .prepare(port.initial().key, &doc, 0, false, 2000)
            .unwrap()
            .with_restarted_provider(sample.at.clock)
            .unwrap()
    };
    submit(&mut backend, authority, 1, MediaCommand::Play, 1000);
    assert!(matches!(
        port.stage_requested(current(&backend).ticket, plan(), sample, map),
        Err(Code::State)
    ));
    submit(
        &mut backend,
        authority,
        2,
        MediaCommand::Recover { position_ms: 0 },
        1001,
    );
    let request = current(&backend);
    let ordinary = prepare
        .prepare(port.initial().key, &doc, 0, false, 2000)
        .unwrap();
    assert!(matches!(
        port.stage_requested(request.ticket, ordinary, sample, map),
        Err(Code::State)
    ));
    port.complete_control(request.ticket, Ok(())).unwrap();
    backend.tick(1002).unwrap();
    assert_eq!(
        backend.state().media[0].unwrap().control.unwrap().result,
        Some(Err(ControlFailure::Provider(Code::State)))
    );
    assert_eq!(
        submit(
            &mut backend,
            authority,
            3,
            MediaCommand::Recover { position_ms: 1000 },
            1003
        )
        .result,
        Err(Code::State)
    );
    backend
        .acquire(
            Grant {
                principal: [2; 16],
                origin: Origin::Remote,
                duration_ms: 1000,
            },
            true,
            1004,
        )
        .unwrap();
    assert!(
        backend
            .submit(
                stagemaster_runtime::Request {
                    lease: authority,
                    serial: 4,
                    expected_revision: backend.state().revision,
                    action: crate::Action::RequestMedia {
                        group: port.initial().key,
                        command: MediaCommand::Recover { position_ms: 0 }
                    }
                },
                1005
            )
            .is_err()
    );
    assert_eq!(
        backend.state().media[0].unwrap().group.key,
        port.initial().key
    );
}

#[test]
fn replaced_and_expired_recovery_plans_are_reclaimed_without_rebinding() {
    for expire in [false, true] {
        let (mut backend, port, prepare, doc, original, old_map) = setup_controlled(true);
        let authority = lease(&mut backend);
        let (sample, map) = rebound(original, old_map);
        let plan = prepare
            .prepare(port.initial().key, &doc, 0, false, 2000)
            .unwrap()
            .with_restarted_provider(sample.at.clock)
            .unwrap();
        submit(
            &mut backend,
            authority,
            1,
            MediaCommand::Recover { position_ms: 0 },
            1000,
        );
        let request = current(&backend);
        let staged = port
            .stage_requested(request.ticket, plan, sample, map)
            .unwrap();
        let guard = port.staging.lock().unwrap();
        if expire {
            backend.tick(1051).unwrap();
        } else {
            submit(&mut backend, authority, 2, MediaCommand::Stop, 1001);
        }
        drop(guard);
        backend.tick(1052).unwrap();
        assert_eq!(
            port.reclaim(staged, false).unwrap().result,
            Some(Err(Code::State))
        );
        assert_eq!(
            port.complete_control(request.ticket, Ok(())),
            Err(Code::State)
        );
        assert_eq!(
            backend.state().media[0].unwrap().group.provider,
            original.at.clock
        );
        assert_eq!(
            backend.state().media[0].unwrap().group.key,
            port.initial().key
        );
    }
}
