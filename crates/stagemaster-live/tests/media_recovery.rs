#[path = "support/media.rs"]
mod media_support;
mod support;
use media_support::*;
use stagemaster_live::media::{Sample, Status};
use stagemaster_time::{Clock, Exchange, Mapping};

fn restarted() -> Clock {
    Clock::new(provider().id(), provider().epoch() + 1).unwrap()
}
fn new_mapping(clock: Clock, host: Clock, at_ms: u64) -> Mapping {
    Mapping::measure(
        Exchange {
            sent: clock.at(0),
            received: clock.at(0),
            received_at_target: host.at(at_ms * 1_000_000),
            sent_at_target: host.at(at_ms * 1_000_000),
        },
        stagemaster_time::Limits {
            max_round_trip_ns: 1,
            max_age_ns: 10_000_000_000,
            max_uncertainty_ns: 1,
            relative_drift_ppm: 0,
            timestamp_error_ns: 0,
        },
    )
    .unwrap()
}
fn paused(clock: Clock) -> Sample {
    Sample {
        at: clock.at(0),
        sequence: 1,
        position_ms: 0,
        playing: false,
        progress: None,
    }
}

#[test]
fn restart_rebinds_only_the_failed_group_and_rejects_old_plans_clocks_and_keys() {
    let (doc, mut session, old_map) = setup();
    start(&doc, &mut session, &old_map, 0, 1000);
    let old_key = session.media_key(GROUP).unwrap();
    let prepare = session.media_preparer(old_key).unwrap();
    let mut late = prepare.prepare(old_key, &doc, 0, false, 4000).unwrap();
    session.tick(2101).unwrap();
    assert_eq!(session.media_groups().next().unwrap().status, Status::Lost);
    let autonomous_before = session.values().unwrap()[3];
    let clock = restarted();
    let map = new_mapping(clock, session.host_clock(), 2101);
    let mut recovery = prepare
        .prepare(old_key, &doc, 0, false, 2200)
        .unwrap()
        .with_restarted_provider(clock)
        .unwrap();
    session
        .activate_media(&mut recovery, paused(clock), &map, 2101)
        .unwrap();
    let group = session.media_groups().next().unwrap();
    assert_eq!(group.provider, clock);
    assert_eq!(group.status, Status::Paused);
    assert_ne!(group.key, old_key);
    assert_eq!(session.values().unwrap()[3], autonomous_before);
    let before = *session.frame().unwrap();
    assert!(
        session
            .activate_media(&mut late, sample(1, 0, false, 2101), &old_map, 2102)
            .is_err()
    );
    assert!(
        session
            .observe_media(old_key, paused(clock), &map, 2102)
            .is_err()
    );
    assert!(
        session
            .observe_media(group.key, sample(2, 0, false, 2101), &old_map, 2102)
            .is_err()
    );
    assert!(
        session
            .observe_media(group.key, paused(clock), &old_map, 2102)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    session
        .observe_media(
            group.key,
            Sample {
                at: clock.at(100_000_000),
                sequence: 2,
                position_ms: 100,
                playing: true,
                progress: None,
            },
            &map,
            2201,
        )
        .unwrap();
    assert_eq!(
        session.media_groups().next().unwrap().status,
        Status::Following
    );
    assert_ne!(session.values().unwrap()[3], autonomous_before);
    assert_frame(&doc, &session, session.values().unwrap());
}

#[test]
fn bad_restart_identity_epoch_mapping_and_deadline_leave_execution_unchanged() {
    let (doc, mut session, _) = setup();
    let key = session.media_key(GROUP).unwrap();
    let prepare = session.media_preparer(key).unwrap();
    let before = *session.frame().unwrap();
    for clock in [
        provider(),
        Clock::new(provider().id(), 6).unwrap(),
        Clock::new([90; 16], 8).unwrap(),
    ] {
        let mut prepared = prepare
            .prepare(key, &doc, 0, false, 2000)
            .unwrap()
            .with_restarted_provider(clock)
            .unwrap();
        let map = new_mapping(clock, session.host_clock(), 1000);
        assert!(
            session
                .activate_media(&mut prepared, paused(clock), &map, 1001)
                .is_err()
        );
        assert_eq!(session.frame(), Some(&before));
        assert_eq!(session.media_groups().next().unwrap().provider, provider());
    }
    let clock = restarted();
    let wrong_map = new_mapping(provider(), session.host_clock(), 1000);
    let mut prepared = prepare
        .prepare(key, &doc, 0, false, 2000)
        .unwrap()
        .with_restarted_provider(clock)
        .unwrap();
    assert!(
        session
            .activate_media(&mut prepared, paused(clock), &wrong_map, 1001)
            .is_err()
    );
    let map = new_mapping(clock, session.host_clock(), 1000);
    let mut expired = prepare
        .prepare(key, &doc, 0, false, 1001)
        .unwrap()
        .with_restarted_provider(clock)
        .unwrap();
    assert!(
        session
            .activate_media(&mut expired, paused(clock), &map, 1001)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    assert_eq!(session.media_key(GROUP), Some(key));
    assert!(session.fault().is_none());
}

#[test]
fn normal_start_cannot_switch_clocks_and_recovery_cannot_start_audibly() {
    let (doc, mut session, _) = setup();
    let key = session.media_key(GROUP).unwrap();
    let prepare = session.media_preparer(key).unwrap();
    let clock = restarted();
    assert!(
        prepare
            .prepare(key, &doc, 0, true, 2000)
            .unwrap()
            .with_restarted_provider(clock)
            .is_err()
    );
    let mut ordinary = prepare.prepare(key, &doc, 0, false, 2000).unwrap();
    let map = new_mapping(clock, session.host_clock(), 1000);
    let before = *session.frame().unwrap();
    assert!(
        session
            .activate_media(&mut ordinary, paused(clock), &map, 1001)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    assert_eq!(session.media_key(GROUP), Some(key));
}
