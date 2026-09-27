use stagemaster_playback::{Curve, EffectChannel, Plan, Player, Status, Step};

fn channel(index: usize, curve: Curve, phase: u16) -> EffectChannel {
    EffectChannel {
        index,
        low: 0,
        high: 65535,
        period_ms: 1000,
        phase,
        curve,
        duty_percent: 25,
    }
}
fn step(target: u16, delay_ms: u64, fade_ms: u64, wait_ms: Option<u64>) -> Step {
    Step {
        target: vec![target],
        delay_ms,
        fade_ms,
        wait_ms,
    }
}
fn player(curve: Curve) -> Player {
    Player::new(
        Plan::with_effects(
            vec![42],
            vec![step(30000, 0, 0, None)],
            false,
            vec![vec![channel(0, curve, 0)]],
        )
        .unwrap(),
        0,
    )
}
#[test]
fn absolute_curves_have_exact_endpoints_and_wrap_without_drift() {
    for curve in [Curve::Smooth, Curve::Triangle] {
        let mut p = player(curve);
        p.execute(0, 0).unwrap();
        assert_eq!(p.values(), [0]);
        p.advance(250).unwrap();
        assert_eq!(p.values(), [32768]);
        p.advance(500).unwrap();
        assert_eq!(p.values(), [65535]);
        p.advance(1000).unwrap();
        assert_eq!(p.values(), [0]);
        p.advance(1_000_000_500).unwrap();
        assert_eq!(p.values(), [65535]);
        p.stop(1_000_000_501).unwrap();
        assert_eq!(p.values(), [42]);
    }
}
#[test]
fn smooth_has_gentler_ends_and_downward_color_channels_work() {
    let mut p = player(Curve::Smooth);
    p.execute(0, 0).unwrap();
    p.advance(125).unwrap();
    assert_eq!(p.values(), [10240]);
    let mut c = channel(0, Curve::Triangle, 0);
    c.low = 65535;
    c.high = 0;
    let mut p = Player::new(
        Plan::with_effects(vec![0], vec![step(0, 0, 0, None)], false, vec![vec![c]]).unwrap(),
        0,
    );
    p.execute(0, 0).unwrap();
    assert_eq!(p.values(), [65535]);
    p.advance(500).unwrap();
    assert_eq!(p.values(), [0]);
}
#[test]
fn phase_is_a_delay_and_pulse_boundary_does_not_light_two_lamps() {
    let channels = (0..4)
        .map(|i| channel(i, Curve::Pulse, u16::try_from(i * 16384).unwrap()))
        .collect();
    let plan = Plan::with_effects(
        vec![0; 4],
        vec![Step {
            target: vec![0; 4],
            delay_ms: 0,
            fade_ms: 0,
            wait_ms: None,
        }],
        false,
        vec![channels],
    )
    .unwrap();
    let mut p = Player::new(plan, 0);
    p.execute(0, 0).unwrap();
    for i in 0..4 {
        p.advance(i * 250).unwrap();
        assert_eq!(p.values().iter().filter(|v| **v == 65535).count(), 1);
        assert_eq!(p.values()[usize::try_from(i).unwrap()], 65535);
    }
}
#[test]
fn pause_resume_freezes_phase_and_reexecute_resets_it() {
    let mut p = player(Curve::Triangle);
    p.execute(0, 0).unwrap();
    p.pause(125).unwrap();
    let frozen = p.values()[0];
    p.advance(10_000).unwrap();
    assert_eq!(p.values()[0], frozen);
    p.resume(20_000).unwrap();
    p.advance(20_125).unwrap();
    assert_eq!(p.values(), [32768]);
    assert!(p.advance(19_000).is_err());
    assert_eq!(p.values(), [32768]);
    p.execute(0, 20_125).unwrap();
    assert_eq!(p.values(), [0]);
}
#[test]
fn delay_freezes_outgoing_value_and_fade_blends_the_dynamic_target() {
    let plan = Plan::with_effects(
        vec![0],
        vec![step(65535, 0, 0, None), step(0, 100, 1000, None)],
        false,
        vec![vec![], vec![channel(0, Curve::Triangle, 0)]],
    )
    .unwrap();
    let mut p = Player::new(plan, 0);
    p.execute(0, 0).unwrap();
    p.next(100).unwrap();
    p.advance(199).unwrap();
    assert_eq!(p.values(), [65535]);
    p.advance(450).unwrap();
    assert_eq!(p.values(), [57343]);
    p.advance(1200).unwrap();
    assert_eq!(p.values(), [0]);
}
#[test]
fn automatic_boundaries_and_late_frames_equal_frequent_updates() {
    let plan = Plan::with_effects(
        vec![123],
        vec![step(0, 100, 200, Some(75)), step(0, 50, 175, Some(25))],
        true,
        vec![
            vec![channel(0, Curve::Triangle, 9000)],
            vec![channel(0, Curve::Smooth, 30000)],
        ],
    )
    .unwrap();
    let mut fine = Player::new(plan.clone(), 0);
    let mut late = Player::new(plan, 0);
    fine.execute(0, 0).unwrap();
    late.execute(0, 0).unwrap();
    for now in 1..=10437 {
        fine.advance(now).unwrap();
    }
    late.advance(10437).unwrap();
    assert_eq!(
        (fine.values(), fine.index(), fine.elapsed_ms()),
        (late.values(), late.index(), late.elapsed_ms())
    );
    fine.next(10437).unwrap();
    late.next(10437).unwrap();
    fine.advance(10600).unwrap();
    late.advance(10600).unwrap();
    assert_eq!(fine.values(), late.values());
}
#[test]
fn finished_freezes_exact_boundary_and_next_static_step_has_no_effect_tracking() {
    let plan = Plan::with_effects(
        vec![0],
        vec![step(0, 0, 0, Some(250))],
        false,
        vec![vec![channel(0, Curve::Triangle, 0)]],
    )
    .unwrap();
    let mut p = Player::new(plan, 0);
    p.execute(0, 0).unwrap();
    p.advance(10000).unwrap();
    assert_eq!(p.status(), Status::Finished);
    assert_eq!(p.values(), [32768]);
    p.advance(11000).unwrap();
    assert_eq!(p.values(), [32768]);
    let plan = Plan::with_effects(
        vec![0],
        vec![step(0, 0, 0, Some(250)), step(1234, 0, 1000, None)],
        false,
        vec![vec![channel(0, Curve::Triangle, 0)], vec![]],
    )
    .unwrap();
    let mut p = Player::new(plan, 0);
    p.execute(0, 0).unwrap();
    p.advance(250).unwrap();
    assert_eq!(p.values(), [32768]);
    p.advance(1250).unwrap();
    assert_eq!(p.values(), [1234]);
}
#[test]
fn invalid_effect_plans_are_rejected_before_execution() {
    for mut c in [channel(0, Curve::Pulse, 0), channel(0, Curve::Triangle, 0)] {
        c.period_ms = 0;
        assert!(
            Plan::with_effects(vec![0], vec![step(0, 0, 0, None)], false, vec![vec![c]]).is_err()
        );
    }
    for channels in [
        vec![channel(1, Curve::Smooth, 0)],
        vec![channel(0, Curve::Smooth, 0); 2],
    ] {
        assert!(
            Plan::with_effects(vec![0], vec![step(0, 0, 0, None)], false, vec![channels]).is_err()
        );
    }
    let steps = vec![
        Step {
            target: vec![0; 512],
            delay_ms: 0,
            fade_ms: 0,
            wait_ms: None
        };
        33
    ];
    let channels = (0..512)
        .map(|i| channel(i, Curve::Smooth, 0))
        .collect::<Vec<_>>();
    assert!(Plan::with_effects(vec![0; 512], steps, false, vec![channels; 33]).is_err());
}
