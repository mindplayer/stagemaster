use stagemaster_playback::{Plan, Player, Status, Step};
fn step(value: u16, delay: u64, fade: u64, wait: Option<u64>) -> Step {
    Step {
        target: vec![value],
        delay_ms: delay,
        fade_ms: fade,
        wait_ms: wait,
    }
}
fn player(steps: Vec<Step>, repeat: bool) -> Player {
    Player::new(Plan::new(vec![1000], steps, repeat).unwrap(), 0)
}
#[test]
fn delay_fade_wait_finish_and_release_have_independent_meanings() {
    let mut p = player(vec![step(5000, 500, 1000, Some(300))], false);
    p.execute(0, 0).unwrap();
    for (t, expected, status) in [
        (499, 1000, Status::Running),
        (500, 1000, Status::Running),
        (1000, 3000, Status::Running),
        (1500, 5000, Status::Running),
        (1799, 5000, Status::Running),
        (1800, 5000, Status::Finished),
    ] {
        p.advance(t).unwrap();
        assert_eq!(p.values(), [expected]);
        assert_eq!(p.status(), status);
    }
    p.stop(1800).unwrap();
    assert_eq!(p.values(), [1000]);
    assert_eq!(p.status(), Status::Idle);
}
#[test]
fn pause_freezes_delay_fade_and_follow_wait() {
    for at in [100, 700, 1700] {
        let mut p = player(
            vec![step(5000, 500, 1000, Some(300)), step(9000, 0, 0, None)],
            false,
        );
        p.execute(0, 0).unwrap();
        p.pause(at).unwrap();
        let values = p.values().to_vec();
        let elapsed = p.elapsed_ms();
        p.advance(at + 10_000).unwrap();
        assert_eq!(p.values(), values);
        assert_eq!(p.elapsed_ms(), elapsed);
        p.resume(at + 10_000).unwrap();
        p.advance(11_800).unwrap();
        assert_eq!(p.index(), Some(1));
        assert_eq!(p.values(), [9000]);
    }
}
#[test]
fn interrupted_fade_starts_at_current_value_and_invalid_commands_are_atomic() {
    let mut p = player(
        vec![step(5000, 0, 1000, None), step(1000, 0, 1000, None)],
        false,
    );
    p.execute(0, 0).unwrap();
    p.execute(1, 500).unwrap();
    assert_eq!(p.values(), [3000]);
    p.advance(1000).unwrap();
    assert_eq!(p.values(), [2000]);
    assert!(p.execute(999, 2000).is_err());
    assert_eq!(p.values(), [2000]);
    assert!(p.advance(999).is_err());
    assert_eq!(p.elapsed_ms(), 500);
    p.advance(1500).unwrap();
    assert_eq!(p.values(), [1000]);
}
#[test]
fn long_jump_and_fine_steps_match_across_loops_and_initial_entry() {
    let steps = vec![
        step(5000, 50, 100, Some(50)),
        step(10000, 0, 200, Some(100)),
    ];
    let mut a = player(steps.clone(), true);
    let mut b = player(steps, true);
    a.execute(0, 0).unwrap();
    b.execute(0, 0).unwrap();
    for t in 1..=12_345 {
        a.advance(t).unwrap();
    }
    b.advance(12_345).unwrap();
    assert_eq!(a.values(), b.values());
    assert_eq!(a.index(), b.index());
    assert_eq!(a.elapsed_ms(), b.elapsed_ms());
    b.advance(500_000_000_345).unwrap();
    assert_eq!(b.index(), Some(1));
    assert_eq!(b.values(), [8625]);
}
#[test]
fn zero_duration_steps_are_bounded_and_manual_wait_breaks_cycles() {
    assert!(Plan::new(vec![0], vec![step(0, 0, 0, Some(0))], true).is_err());
    let mut p = player(
        vec![step(2000, 0, 0, Some(0)), step(3000, 0, 0, None)],
        true,
    );
    p.execute(0, 0).unwrap();
    assert_eq!(p.index(), Some(1));
    assert_eq!(p.values(), [3000]);
    p.advance(u64::MAX).unwrap();
    assert_eq!(p.index(), Some(1));
    p.next(u64::MAX).unwrap();
    assert_eq!(p.index(), Some(1));
}
#[test]
fn automatic_boundary_is_applied_before_same_time_manual_command() {
    let mut p = player(
        vec![
            step(2000, 0, 0, Some(100)),
            step(3000, 0, 0, None),
            step(4000, 0, 0, None),
        ],
        false,
    );
    p.execute(0, 0).unwrap();
    p.next(100).unwrap();
    assert_eq!(p.index(), Some(2));
    assert_eq!(p.values(), [4000]);
}
#[test]
fn integer_interpolation_rounds_both_directions_and_respects_nonzero_defaults() {
    let mut p = player(vec![step(0, 0, 3, None), step(65535, 0, 2, None)], false);
    p.execute(0, 0).unwrap();
    p.advance(1).unwrap();
    assert_eq!(p.values(), [667]);
    p.advance(3).unwrap();
    p.execute(1, 3).unwrap();
    p.advance(4).unwrap();
    assert_eq!(p.values(), [32768]);
}
#[test]
fn plan_limits_are_enforced_before_running() {
    assert!(Plan::new(vec![], vec![], false).is_err());
    assert!(Plan::new(vec![0], vec![step(0, 0, 86_400_001, None)], false).is_err());
    assert!(Plan::new(vec![0, 0], vec![step(0, 0, 0, None)], false).is_err());
    assert!(Plan::new(vec![0], vec![step(0, 0, 0, None); 1025], false).is_err());
    assert!(
        Plan::new(
            vec![0; 512],
            vec![
                Step {
                    target: vec![0; 512],
                    delay_ms: 0,
                    fade_ms: 0,
                    wait_ms: None
                };
                513
            ],
            false
        )
        .is_err()
    );
    let p = Plan::new(vec![0], vec![step(0, 0, 0, None)], false).unwrap();
    assert_eq!(p.value_buffer_bytes(), 8);
}

#[test]
fn advancing_past_last_step_never_freezes_an_unfinished_fade() {
    let mut p = player(vec![step(5000, 0, 1000, None)], false);
    assert!(p.can_next());
    p.execute(0, 0).unwrap();
    assert!(!p.can_next());
    p.next(500).unwrap();
    assert_eq!(p.values(), [3000]);
    assert_eq!(p.status(), Status::Running);
    p.advance(1000).unwrap();
    assert_eq!(p.values(), [5000]);
}
