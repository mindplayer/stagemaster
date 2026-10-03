use stagemaster_playback::{Curve, EffectChannel, HeldScene, MAX_TIME_MS, Plan, Player, Step};
fn plan() -> Plan {
    Plan::with_snap_attributes(
        vec![50_000, 123],
        vec![Step {
            target: vec![0, 456],
            delay_ms: 0,
            fade_ms: 501,
            wait_ms: None,
        }],
        false,
        vec![vec![EffectChannel {
            index: 0,
            low: 1000,
            high: 65_000,
            period_ms: 997,
            phase: 12345,
            curve: Curve::Triangle,
            duty_percent: 50,
        }]],
        vec![1],
    )
    .unwrap()
    .with_effect_time_offset(1733)
    .unwrap()
    .with_entry_fade_offset(211)
    .unwrap()
}
#[test]
fn random_positions_use_the_original_interpolation_effect_and_discrete_semantics() {
    let plan = plan();
    let mut sampler = HeldScene::new(plan.clone()).unwrap();
    for time in [0, 1, 99, 289, 290, 500, 501, 2000, MAX_TIME_MS, 501, 0] {
        let mut reference = Player::new(plan.clone(), 0);
        reference.execute(0, 0).unwrap();
        reference.advance(time).unwrap();
        assert_eq!(sampler.sample(time).unwrap(), reference.values());
    }
    let before = sampler.values().to_vec();
    assert!(sampler.sample(MAX_TIME_MS + 1).is_err());
    assert_eq!(sampler.values(), before);
}
#[test]
fn automatic_delayed_repeating_and_multi_step_plans_cannot_be_sampled_as_one_scene() {
    for (delay, wait, repeat, count) in [
        (1, None, false, 1),
        (0, Some(1), false, 1),
        (0, None, true, 1),
        (0, None, false, 2),
    ] {
        let steps = vec![
            Step {
                target: vec![1],
                delay_ms: delay,
                fade_ms: 0,
                wait_ms: wait
            };
            count
        ];
        let plan = Plan::new(vec![0], steps, repeat).unwrap();
        assert!(HeldScene::new(plan).is_err());
    }
}
