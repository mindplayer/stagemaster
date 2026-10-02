use stagemaster_playback::{
    CrossfadeTiming, Curve, EffectChannel, MAX_ATTRIBUTES, MAX_TIME_MS, Plan, Player,
    SceneCrossfade, Step,
};

fn moving(period: u32, snap: u16) -> Plan {
    Plan::with_snap_attributes(
        vec![0, 111],
        vec![Step {
            target: vec![0, snap],
            delay_ms: 0,
            fade_ms: 0,
            wait_ms: None,
        }],
        false,
        vec![vec![EffectChannel {
            index: 0,
            low: 0,
            high: u16::MAX,
            period_ms: period,
            phase: 0,
            curve: Curve::Triangle,
            duty_percent: 50,
        }]],
        vec![1],
    )
    .unwrap()
}
fn timing() -> CrossfadeTiming {
    CrossfadeTiming {
        duration_ms: 1000,
        offset_ms: 0,
        source_elapsed_ms: 0,
        target_elapsed_ms: 0,
    }
}
fn at(plan: &Plan, time: u64) -> Vec<u16> {
    let mut player = Player::new(plan.clone(), 0);
    player.execute(0, 0).unwrap();
    player.advance(time).unwrap();
    player.values().to_vec()
}

#[test]
fn both_effects_keep_running_and_discrete_values_switch_at_entry() {
    let mut sampler = SceneCrossfade::new(moving(1000, 222), moving(2000, 555), timing()).unwrap();
    assert_eq!(sampler.values(), [0, 555]);
    // Triangle sources at 250 ms: 32768 and 16384. Freezing the outgoing source at zero is wrong.
    assert_eq!(sampler.sample(250).unwrap(), [28672, 555]);
    assert_eq!(sampler.sample(500).unwrap(), [49152, 555]);
    assert_eq!(sampler.sample(1000).unwrap(), [65535, 555]);
    assert_eq!(sampler.sample(1500).unwrap(), [32768, 555]);
    assert_eq!(sampler.sample(0).unwrap(), [0, 555]);
}

#[test]
fn isolated_source_fades_and_effect_offsets_match_the_existing_player() {
    let base = moving(733, 123);
    let mut steps = base.steps().to_vec();
    steps[0].fade_ms = 1379;
    let source = Plan::with_snap_attributes(
        vec![45000, 88],
        steps,
        false,
        base.effects().to_vec(),
        vec![1],
    )
    .unwrap()
    .with_effect_time_offset(333)
    .unwrap()
    .with_entry_fade_offset(201)
    .unwrap();
    let target = moving(1199, 987).with_effect_time_offset(59).unwrap();
    let clocks = CrossfadeTiming {
        duration_ms: 1201,
        offset_ms: 89,
        source_elapsed_ms: 67,
        target_elapsed_ms: 211,
    };
    let mut sampler = SceneCrossfade::new(source.clone(), target.clone(), clocks).unwrap();
    for t in [0, 1, 100, 578, 1111, 1112, 1200, 2400, 500, 0] {
        let a = at(&source, clocks.source_elapsed_ms + t);
        let b = at(&target, clocks.target_elapsed_ms + t);
        let progress = (clocks.offset_ms + t).min(clocks.duration_ms);
        let expected = (u64::from(a[0]) * (clocks.duration_ms - progress)
            + u64::from(b[0]) * progress
            + clocks.duration_ms / 2)
            / clocks.duration_ms;
        assert_eq!(
            sampler.sample(t).unwrap(),
            [u16::try_from(expected).unwrap(), b[1]]
        );
    }
}

#[test]
fn both_scenes_keep_their_own_entry_fade_clocks() {
    let scene = |from, to, duration, offset| {
        Plan::new(
            vec![from],
            vec![Step {
                target: vec![to],
                delay_ms: 0,
                fade_ms: duration,
                wait_ms: None,
            }],
            false,
        )
        .unwrap()
        .with_entry_fade_offset(offset)
        .unwrap()
    };
    let mut sampler = SceneCrossfade::new(
        scene(0, 60000, 2000, 500),
        scene(10000, 50000, 1000, 200),
        CrossfadeTiming {
            duration_ms: 1000,
            offset_ms: 250,
            source_elapsed_ms: 250,
            target_elapsed_ms: 100,
        },
    )
    .unwrap();
    assert_eq!(sampler.values(), [22375]);
    // At local 250 ms the two independent scene fades yield 30000 / 32000; outer progress is 50%.
    assert_eq!(sampler.sample(250).unwrap(), [31000]);
}

#[test]
fn sliced_transition_and_scrubbing_match_every_original_sample() {
    let source = moving(977, 44).with_effect_time_offset(197).unwrap();
    let target = moving(1289, 55).with_effect_time_offset(731).unwrap();
    let clocks = CrossfadeTiming {
        duration_ms: 1201,
        source_elapsed_ms: 99,
        target_elapsed_ms: 23,
        offset_ms: 71,
    };
    let mut original = SceneCrossfade::new(source.clone(), target.clone(), clocks).unwrap();
    for cut in [1, 333, 1129, 1130, 1300] {
        let sliced = CrossfadeTiming {
            source_elapsed_ms: clocks.source_elapsed_ms + cut,
            target_elapsed_ms: clocks.target_elapsed_ms + cut,
            offset_ms: clocks.offset_ms + cut,
            ..clocks
        };
        let mut right = SceneCrossfade::new(source.clone(), target.clone(), sliced).unwrap();
        for local in (0..1500).rev() {
            assert_eq!(
                right.sample(local).unwrap(),
                original.sample(cut + local).unwrap()
            );
        }
    }
}

#[test]
fn plan_and_clock_mismatches_are_rejected() {
    let source = moving(1000, 11);
    let target = moving(2000, 22);
    for mut clocks in [timing(); 5].into_iter().enumerate() {
        match clocks.0 {
            0 => clocks.1.duration_ms = 0,
            1 => clocks.1.duration_ms = MAX_TIME_MS + 1,
            2 => clocks.1.source_elapsed_ms = MAX_TIME_MS + 1,
            3 => clocks.1.target_elapsed_ms = MAX_TIME_MS + 1,
            _ => clocks.1.offset_ms = MAX_TIME_MS + 1,
        }
        assert!(SceneCrossfade::new(source.clone(), target.clone(), clocks.1).is_err());
    }
    let one = Plan::new(
        vec![0],
        vec![Step {
            target: vec![0],
            delay_ms: 0,
            fade_ms: 0,
            wait_ms: None,
        }],
        false,
    )
    .unwrap();
    assert!(SceneCrossfade::new(one, target.clone(), timing()).is_err());
    let different_mask = Plan::new(
        vec![0; 2],
        vec![Step {
            target: vec![0; 2],
            delay_ms: 0,
            fade_ms: 0,
            wait_ms: None,
        }],
        false,
    )
    .unwrap();
    assert!(SceneCrossfade::new(different_mask, target.clone(), timing()).is_err());
    for index in 0..4 {
        let mut steps = source.steps().to_vec();
        match index {
            0 => steps[0].delay_ms = 1,
            1 => steps[0].wait_ms = Some(1),
            2 => steps.push(steps[0].clone()),
            _ => {}
        }
        let unsupported = Plan::with_snap_attributes(
            vec![0; 2],
            steps.clone(),
            index == 3,
            vec![vec![]; steps.len()],
            vec![1],
        )
        .unwrap();
        assert!(SceneCrossfade::new(unsupported.clone(), target.clone(), timing()).is_err());
        assert!(SceneCrossfade::new(target.clone(), unsupported, timing()).is_err());
    }
}

#[test]
fn maximum_attributes_and_invalid_sample_keep_the_previous_buffer() {
    let plan = |value| {
        Plan::new(
            vec![value; MAX_ATTRIBUTES],
            vec![Step {
                target: vec![value; MAX_ATTRIBUTES],
                delay_ms: 0,
                fade_ms: 0,
                wait_ms: None,
            }],
            false,
        )
        .unwrap()
    };
    let mut sampler = SceneCrossfade::new(
        plan(0),
        plan(u16::MAX),
        CrossfadeTiming {
            duration_ms: MAX_TIME_MS,
            ..timing()
        },
    )
    .unwrap();
    let address = sampler.values().as_ptr();
    for t in [0, MAX_TIME_MS / 2, MAX_TIME_MS] {
        assert_eq!(sampler.sample(t).unwrap().as_ptr(), address);
    }
    assert_eq!(sampler.values(), vec![u16::MAX; MAX_ATTRIBUTES]);
    for bad in [MAX_TIME_MS + 1, u64::MAX] {
        assert!(sampler.sample(bad).is_err());
        assert_eq!(sampler.values(), vec![u16::MAX; MAX_ATTRIBUTES]);
    }
    for clocks in [
        CrossfadeTiming {
            source_elapsed_ms: MAX_TIME_MS,
            ..timing()
        },
        CrossfadeTiming {
            target_elapsed_ms: MAX_TIME_MS,
            ..timing()
        },
        CrossfadeTiming {
            offset_ms: MAX_TIME_MS,
            ..timing()
        },
    ] {
        let mut edge = SceneCrossfade::new(plan(1), plan(2), clocks).unwrap();
        let before = edge.values().to_vec();
        assert!(edge.sample(1).is_err());
        assert_eq!(edge.values(), before);
    }
}
