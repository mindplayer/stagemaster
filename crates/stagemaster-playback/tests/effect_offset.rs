use stagemaster_playback::{
    Curve, EffectChannel, Keyframe, MAX_TIME_MS, Plan, Player, Step, Transition,
};
fn plan(curve: Curve, period: u32, fade: u64) -> Plan {
    Plan::with_snap_attributes(
        vec![1000, 1234],
        vec![Step {
            target: vec![2000, 60000],
            delay_ms: 0,
            fade_ms: fade,
            wait_ms: None,
        }],
        false,
        vec![vec![EffectChannel {
            index: 0,
            low: 3000,
            high: 60000,
            period_ms: period,
            phase: 919,
            curve,
            duty_percent: 37,
        }]],
        vec![1],
    )
    .unwrap()
}
fn sample(plan: Plan, elapsed: u64) -> Vec<u16> {
    let mut player = Player::new(plan, 0);
    player.execute(0, 0).unwrap();
    player.advance(elapsed).unwrap();
    player.values().to_vec()
}
#[test]
fn integer_source_offset_preserves_all_curves_without_phase_angle_rounding() {
    let keyframes = Curve::Keyframes(vec![
        Keyframe {
            phase: 0,
            value: 100,
            transition: Transition::Linear,
        },
        Keyframe {
            phase: 21000,
            value: 58000,
            transition: Transition::Smooth,
        },
        Keyframe {
            phase: 50000,
            value: 4500,
            transition: Transition::Hold,
        },
    ]);
    for curve in [Curve::Smooth, Curve::Triangle, Curve::Pulse, keyframes] {
        for period in [101, 997, 1601] {
            let p = plan(curve.clone(), period, 0);
            for offset in [1, 333, 1999, MAX_TIME_MS] {
                let shifted = p.clone().with_effect_time_offset(offset).unwrap();
                for t in (0..7000).step_by(37) {
                    assert_eq!(
                        sample(shifted.clone(), t),
                        sample(p.clone(), t + offset),
                        "period {period}, offset {offset}, time {t}"
                    );
                }
                let expected =
                    (u64::MAX % u64::from(period) + offset % u64::from(period)) % u64::from(period);
                assert_eq!(sample(shifted, u64::MAX), sample(p.clone(), expected));
            }
        }
    }
}
#[test]
fn entry_fade_and_discrete_values_keep_local_time_while_effect_source_advances() {
    let source = plan(Curve::Triangle, 997, 0);
    let shifted = plan(Curve::Triangle, 997, 1000)
        .with_effect_time_offset(333)
        .unwrap();
    for elapsed in [0, 1, 100, 499, 999, 1000, 2500] {
        let target = u64::from(sample(source.clone(), elapsed + 333)[0]);
        let expected = if elapsed < 1000 {
            (1000 * (1000 - elapsed) + target * elapsed + 500) / 1000
        } else {
            target
        };
        let result = sample(shifted.clone(), elapsed);
        assert_eq!(u64::from(result[0]), expected);
        assert_eq!(result[1], 60000);
    }
}
#[test]
fn source_clock_offset_is_bounded_and_only_for_one_held_scene() {
    let p = plan(Curve::Pulse, 1000, 0);
    assert_eq!(p.effect_time_offset_ms(), 0);
    assert!(p.clone().with_effect_time_offset(MAX_TIME_MS + 1).is_err());
    for (delay, wait, repeat, count) in [
        (1, None, false, 1),
        (0, Some(10), false, 1),
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
        assert!(
            Plan::new(vec![0], steps, repeat)
                .unwrap()
                .with_effect_time_offset(1)
                .is_err()
        );
    }
}
