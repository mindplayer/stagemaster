use stagemaster_playback::{Curve, EffectChannel, Plan, Player, Status, Step};

fn step(target: [u16; 3], delay_ms: u64, fade_ms: u64, wait_ms: Option<u64>) -> Step {
    Step {
        target: target.to_vec(),
        delay_ms,
        fade_ms,
        wait_ms,
    }
}
fn plan(steps: Vec<Step>, repeat: bool) -> Plan {
    let effects = vec![vec![]; steps.len()];
    Plan::with_snap_attributes(vec![100, 1000, 200], steps, repeat, effects, vec![0, 2]).unwrap()
}
#[test]
fn snap_waits_for_delay_but_never_traverses_intermediate_values() {
    let mut p = Player::new(
        plan(vec![step([60000, 5000, 0], 50, 100, Some(20))], false),
        0,
    );
    p.execute(0, 0).unwrap();
    for (t, expected) in [
        (0, [100, 1000, 200]),
        (49, [100, 1000, 200]),
        (50, [60000, 1000, 0]),
        (100, [60000, 3000, 0]),
        (149, [60000, 4960, 0]),
        (150, [60000, 5000, 0]),
        (170, [60000, 5000, 0]),
    ] {
        p.advance(t).unwrap();
        assert_eq!(p.values(), expected, "at {t}");
    }
    assert_eq!(p.status(), Status::Finished);
    p.stop(170).unwrap();
    assert_eq!(p.values(), [100, 1000, 200]);
}
#[test]
fn paused_delay_and_fade_resume_without_replaying_the_switch() {
    for pause in [49, 50, 100] {
        let mut p = Player::new(plan(vec![step([60000, 5000, 0], 50, 100, None)], false), 0);
        p.execute(0, 0).unwrap();
        p.pause(pause).unwrap();
        let held = p.values().to_vec();
        p.advance(10000 + pause).unwrap();
        assert_eq!(p.values(), held);
        assert_eq!(p.elapsed_ms(), pause);
        p.resume(10000 + pause).unwrap();
        p.advance(10150).unwrap();
        assert_eq!(p.values(), [60000, 5000, 0]);
    }
}
#[test]
fn interrupted_fade_preserves_continuous_origin_and_discrete_delay() {
    let mut p = Player::new(
        plan(
            vec![
                step([60000, 5000, 0], 0, 100, None),
                step([123, 1000, 456], 20, 100, None),
            ],
            false,
        ),
        0,
    );
    p.execute(0, 0).unwrap();
    p.execute(1, 50).unwrap();
    assert_eq!(p.values(), [60000, 3000, 0]);
    p.advance(69).unwrap();
    assert_eq!(p.values(), [60000, 3000, 0]);
    p.advance(70).unwrap();
    assert_eq!(p.values(), [123, 3000, 456]);
    p.advance(120).unwrap();
    assert_eq!(p.values(), [123, 2000, 456]);
    assert!(p.execute(5, 200).is_err());
    assert!(p.advance(119).is_err());
    assert_eq!(p.values(), [123, 2000, 456]);
}
#[test]
fn loops_long_skips_and_zero_fade_match_every_millisecond() {
    let p = plan(
        vec![
            step([60000, 5000, 0], 50, 100, Some(50)),
            step([123, 1000, 456], 0, 0, Some(100)),
        ],
        true,
    );
    let mut fine = Player::new(p.clone(), 0);
    let mut sparse = Player::new(p, 0);
    fine.execute(0, 0).unwrap();
    sparse.execute(0, 0).unwrap();
    for t in 1..=12345 {
        fine.advance(t).unwrap();
        assert!([100, 60000, 123].contains(&fine.values()[0]));
        assert!([200, 0, 456].contains(&fine.values()[2]));
    }
    sparse.advance(12345).unwrap();
    assert_eq!(fine.values(), sparse.values());
    assert_eq!(fine.index(), sparse.index());
    assert_eq!(fine.elapsed_ms(), sparse.elapsed_ms());
    sparse.advance(500_000_000_345).unwrap();
    assert_eq!(sparse.values(), [123, 1000, 456]);
}
#[test]
fn snap_indices_are_bounded_and_dynamic_effects_are_rejected() {
    let make = |snap, effects| {
        Plan::with_snap_attributes(
            vec![0; 3],
            vec![step([0; 3], 0, 100, None)],
            false,
            vec![effects],
            snap,
        )
    };
    for indices in [vec![3], vec![65535], vec![0, 0], vec![2, 1]] {
        assert!(make(indices, vec![]).is_err());
    }
    let effect = EffectChannel {
        index: 2,
        low: 0,
        high: 65535,
        period_ms: 1000,
        phase: 0,
        curve: Curve::Smooth,
        duty_percent: 50,
    };
    assert!(make(vec![0, 2], vec![effect.clone()]).is_err());
    assert!(make(vec![0], vec![effect]).is_ok());
    assert_eq!(make(vec![], vec![]).unwrap().snap_buffer_bytes(), 0);
    assert_eq!(make(vec![0, 1, 2], vec![]).unwrap().snap_buffer_bytes(), 6);
}
#[test]
fn maximum_attributes_and_legacy_constructor_keep_their_rules() {
    let step = Step {
        target: vec![65535; 512],
        delay_ms: 0,
        fade_ms: 100,
        wait_ms: None,
    };
    let snap = Plan::with_snap_attributes(
        vec![0; 512],
        vec![step.clone()],
        false,
        vec![vec![]],
        (0..512).collect(),
    )
    .unwrap();
    assert_eq!(snap.snap_buffer_bytes(), 1024);
    let mut p = Player::new(snap, 0);
    p.execute(0, 0).unwrap();
    assert_eq!(p.values(), vec![65535; 512]);
    let legacy = Plan::new(vec![0; 512], vec![step], false).unwrap();
    assert!(legacy.snap_attributes().is_empty());
    let mut p = Player::new(legacy, 0);
    p.execute(0, 0).unwrap();
    p.advance(50).unwrap();
    assert_eq!(p.values(), vec![32768; 512]);
}
