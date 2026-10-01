use stagemaster_playback::{Curve, EffectChannel, MAX_TIME_MS, Plan, Player, Step};
fn plan(curve: Curve) -> Plan {
    Plan::with_snap_attributes(
        vec![60001, 123, 8000],
        vec![Step {
            target: vec![99, 50000, 40000],
            delay_ms: 0,
            fade_ms: 503,
            wait_ms: None,
        }],
        false,
        vec![vec![EffectChannel {
            index: 0,
            low: 999,
            high: 58001,
            period_ms: 997,
            phase: 12345,
            curve,
            duty_percent: 23,
        }]],
        vec![1],
    )
    .unwrap()
}
fn sample(plan: Plan, time: u64) -> Vec<u16> {
    let mut player = Player::new(plan, 0);
    player.execute(0, 0).unwrap();
    player.advance(time).unwrap();
    player.values().to_vec()
}
#[test]
fn slices_keep_original_integer_weights_with_dynamic_targets_and_snap_attributes() {
    for curve in [Curve::Smooth, Curve::Triangle, Curve::Pulse] {
        let original = plan(curve).with_effect_time_offset(1333).unwrap();
        for cut in [1, 237, 502, 503, 504, 3101] {
            let slice = original
                .clone()
                .with_effect_time_offset(1333 + cut)
                .unwrap()
                .with_entry_fade_offset(cut)
                .unwrap();
            for time in (0..1600).step_by(3).chain([0, 1, 502, 503, 504]) {
                assert_eq!(
                    sample(slice.clone(), time),
                    sample(original.clone(), cut + time)
                );
            }
            assert_eq!(sample(slice, u64::MAX)[1..], [50000, 40000]);
        }
    }
}
#[test]
fn fade_clock_is_independent_and_only_available_for_isolated_held_scenes() {
    let original = plan(Curve::Smooth);
    let sliced = original.clone().with_entry_fade_offset(200).unwrap();
    assert_eq!(sliced.effect_time_offset_ms(), 0);
    let mut paused = Player::new(sliced.clone(), 0);
    paused.execute(0, 0).unwrap();
    paused.pause(27).unwrap();
    let values = paused.values().to_vec();
    paused.advance(10000).unwrap();
    assert_eq!(paused.values(), values);
    paused.resume(10000).unwrap();
    paused.advance(10001).unwrap();
    assert_eq!(paused.values(), sample(sliced, 28));
    assert!(
        original
            .clone()
            .with_entry_fade_offset(MAX_TIME_MS + 1)
            .is_err()
    );
    for (delay, wait, repeat, count) in [
        (1, None, false, 1),
        (0, Some(1), false, 1),
        (0, None, true, 1),
        (0, None, false, 2),
    ] {
        let p = Plan::new(
            vec![0],
            vec![
                Step {
                    target: vec![65535],
                    delay_ms: delay,
                    fade_ms: 10,
                    wait_ms: wait
                };
                count
            ],
            repeat,
        )
        .unwrap();
        assert!(p.with_entry_fade_offset(1).is_err());
    }
}
