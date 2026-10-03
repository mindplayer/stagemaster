#[path = "support/media.rs"]
mod media_support;
mod support;
use media_support::*;
use stagemaster_live::{
    Session,
    media::{Sample, Status},
};
use stagemaster_time::Clock;
use support::*;

#[test]
fn stale_duplicate_future_wrong_clock_and_discontinuous_observations_are_atomic() {
    let (doc, mut session, map) = setup();
    start(&doc, &mut session, &map, 0, 1000);
    let key = session.media_key(GROUP).unwrap();
    let good = sample(2, 20, true, 1020);
    session.observe_media(key, good, &map, 1021).unwrap();
    let before = *session.frame().unwrap();
    let info = session.media_groups().next().unwrap();
    let mut wrong_epoch = sample(3, 21, true, 1021);
    wrong_epoch.at.clock = Clock::new(provider().id(), 8).unwrap();
    let cases = [
        (good, 1022),
        (sample(3, 20, true, 1019), 1022),
        (sample(3, 20, true, 1030), 1022),
        (sample(3, 0, true, 1021), 1022),
        (sample(3, 200, true, 1021), 1022),
        (sample(3, 21, true, 1021), 1040),
        (
            Sample {
                sequence: 0,
                ..sample(3, 21, true, 1021)
            },
            1022,
        ),
        (wrong_epoch, 1022),
    ];
    for (sample, now) in cases {
        assert!(
            session.observe_media(key, sample, &map, now).is_err(),
            "{sample:?}"
        );
        assert_eq!(session.frame(), Some(&before));
        assert_eq!(session.media_groups().next().unwrap(), info);
        assert_eq!(session.observed_ms(), 1021);
        assert!(session.fault().is_none());
    }
    let other_host = mapping(Clock::new([8; 16], 0).unwrap(), 200);
    assert!(
        session
            .observe_media(key, sample(3, 21, true, 1021), &other_host, 1022)
            .is_err()
    );
    let wide = mapping(session.host_clock(), 20_000);
    assert!(
        session
            .observe_media(key, sample(3, 200, true, 1200), &wide, 1205)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    // Input rejection/read retries do not renew the loss deadline.
    session.tick(2021).unwrap();
    assert_eq!(session.media_groups().next().unwrap().status, Status::Lost);
}

#[test]
fn preparation_failure_old_generation_and_late_activation_preserve_current_execution() {
    let (doc, mut session, map) = setup();
    start(&doc, &mut session, &map, 0, 1000);
    let key = session.media_key(GROUP).unwrap();
    let before = *session.frame().unwrap();
    let mut prepared = session
        .media_preparer(key)
        .unwrap()
        .prepare(key, &doc, 60, false, 1050)
        .unwrap();
    assert!(
        session
            .activate_media(&mut prepared, sample(1, 60, false, 1050), &map, 1051)
            .is_err()
    );
    let mut mismatch = session
        .media_preparer(key)
        .unwrap()
        .prepare(key, &doc, 60, false, 1100)
        .unwrap();
    assert!(
        session
            .activate_media(&mut mismatch, sample(1, 60, true, 1050), &map, 1051)
            .is_err()
    );
    let mut changed = fixture(0);
    changed["project"]["name"] = serde_json::json!("不同工程版本");
    assert!(
        session
            .media_preparer(key)
            .unwrap()
            .prepare(key, &decode(&changed), 0, true, 1100)
            .is_err()
    );
    assert!(
        session
            .media_preparer(key)
            .unwrap()
            .prepare(key, &doc, u64::MAX, true, 1100)
            .is_err()
    );
    assert_eq!(session.frame(), Some(&before));
    let mut stale = session
        .media_preparer(key)
        .unwrap()
        .prepare(key, &doc, 80, true, 1100)
        .unwrap();
    start(&doc, &mut session, &map, 60, 1060);
    let after = *session.frame().unwrap();
    assert!(
        session
            .activate_media(&mut stale, sample(1, 80, true, 1070), &map, 1071)
            .is_err()
    );
    assert!(
        session
            .observe_media(key, sample(100, 80, true, 1070), &map, 1071)
            .is_err()
    );
    assert!(session.stop_media(key, 1071).is_err());
    assert_eq!(session.frame(), Some(&after));
    assert!(session.fault().is_none());
}

#[test]
fn group_membership_capabilities_and_host_unit_overflow_are_rejected_before_running() {
    let doc = document();
    let mut bad = Vec::new();
    let mut spec = group();
    spec.sources.push([1; 16]);
    bad.push(spec);
    let mut spec = group();
    spec.sources = vec![[3; 16]];
    bad.push(spec);
    let mut spec = group();
    spec.sources = vec![[8; 16]];
    bad.push(spec);
    let mut spec = group();
    spec.id = [0; 16];
    bad.push(spec);
    let mut spec = group();
    spec.clock = Clock::new([9; 16], 7).unwrap();
    bad.push(spec);
    let mut spec = group();
    spec.limits.max_gap_ms = u64::MAX;
    bad.push(spec);
    let mut spec = group();
    spec.limits.max_rate_percent = 0;
    bad.push(spec);
    for spec in bad {
        assert!(Session::prepare_with_media(&doc, [9; 16], &specs(), &[spec], 1000).is_err());
    }
    assert!(
        Session::prepare_with_media(&doc, [9; 16], &specs(), &[group(), group()], 1000).is_err()
    );
    assert!(Session::prepare_with_media(&doc, [9; 16], &specs(), &[group()], u64::MAX).is_err());
    let mut manual = fixture(0);
    manual["lighting"]["sequences"][0]["steps"][0]["advance"] =
        serde_json::json!({"kind":"manual"});
    assert!(
        Session::prepare_with_media(&decode(&manual), [9; 16], &specs(), &[group()], 1000).is_err()
    );
}
