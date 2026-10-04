use stagemaster_playback::{Phase, Plan, Player, Step};
fn step(delay: u64, fade: u64, wait: Option<u64>) -> Step {
    Step {
        target: vec![50_000],
        delay_ms: delay,
        fade_ms: fade,
        wait_ms: wait,
    }
}
fn player(steps: Vec<Step>, repeat: bool) -> Player {
    Player::new(Plan::new(vec![0], steps, repeat).unwrap(), 0)
}
#[test]
fn exact_phase_boundaries_report_remaining_stage_not_total_show_progress() {
    let mut p = player(vec![step(100, 200, Some(300))], false);
    assert_eq!(p.progress().phase, Phase::Idle);
    assert_eq!(p.progress().next_step, None);
    p.execute(0, 0).unwrap();
    for (time, phase, elapsed, duration) in [
        (0, Phase::Delay, 0, Some(100)),
        (99, Phase::Delay, 99, Some(100)),
        (100, Phase::Fade, 0, Some(200)),
        (299, Phase::Fade, 199, Some(200)),
        (300, Phase::Wait, 0, Some(300)),
        (599, Phase::Wait, 299, Some(300)),
        (600, Phase::Finished, 0, None),
    ] {
        p.advance(time).unwrap();
        let before = p.values().to_vec();
        let info = p.progress();
        assert_eq!(
            (info.phase, info.phase_elapsed_ms, info.phase_duration_ms),
            (phase, elapsed, duration)
        );
        assert_eq!(info.elapsed_ms, time);
        assert_eq!(p.progress(), info);
        assert_eq!(p.values(), before);
        assert_eq!(p.elapsed_ms(), time);
    }
    p.stop(601).unwrap();
    assert_eq!(p.progress().elapsed_ms, 0);
    assert_eq!(p.progress().phase, Phase::Idle);
}
#[test]
fn pause_freezes_each_phase_and_does_not_invent_a_hold_deadline() {
    for (at, phase) in [
        (50, Phase::Delay),
        (150, Phase::Fade),
        (350, Phase::Wait),
        (601, Phase::Hold),
    ] {
        let mut p = player(vec![step(100, 200, Some(300)), step(0, 0, None)], false);
        p.execute(0, 0).unwrap();
        p.pause(at).unwrap();
        let frozen = p.progress();
        assert_eq!(frozen.phase, phase);
        p.advance(at + 10_000).unwrap();
        assert_eq!(p.progress(), frozen);
        p.resume(at + 10_000).unwrap();
        p.advance(at + 10_001).unwrap();
        assert_eq!(p.progress().phase_elapsed_ms, frozen.phase_elapsed_ms + 1);
    }
}
#[test]
fn zero_steps_are_skipped_and_next_reports_loop_boundary() {
    let mut p = player(vec![step(0, 0, Some(0)), step(0, 0, None)], true);
    p.execute(0, 0).unwrap();
    assert_eq!(p.index(), Some(1));
    let info = p.progress();
    assert_eq!(info.phase, Phase::Hold);
    assert_eq!(info.next_step, Some(0));
    assert!(info.next_wrap);
    assert_eq!(info.phase_duration_ms, None);
    p.advance(u64::MAX).unwrap();
    assert_eq!(p.progress().phase_elapsed_ms, u64::MAX);
}
#[test]
fn manual_next_and_automatic_cycle_use_original_indices() {
    let mut p = player(vec![step(10, 10, Some(10)), step(0, 10, Some(10))], true);
    p.execute(0, 0).unwrap();
    assert_eq!(p.progress().next_step, Some(1));
    assert!(!p.progress().next_wrap);
    p.advance(45).unwrap();
    assert_eq!(p.progress().phase, Phase::Wait);
    assert!(p.progress().next_wrap);
    p.next(45).unwrap();
    assert_eq!(p.index(), Some(0));
    assert_eq!(p.progress().elapsed_ms, 0);
    p.advance(50_050).unwrap();
    assert_eq!(p.index(), Some(0));
    assert_eq!(p.progress().elapsed_ms, 5);
}
#[test]
fn sliced_entry_fade_reports_only_remaining_time_and_then_holds() {
    for (offset, remaining) in [(0, 200), (50, 150), (200, 0), (500, 0)] {
        let plan = Plan::new(vec![0], vec![step(0, 200, None)], false)
            .unwrap()
            .with_entry_fade_offset(offset)
            .unwrap();
        let mut p = Player::new(plan, 0);
        p.execute(0, 0).unwrap();
        assert_eq!(
            p.progress().phase_duration_ms,
            (remaining > 0).then_some(remaining)
        );
        p.advance(remaining).unwrap();
        assert_eq!(p.progress().phase, Phase::Hold);
        assert_eq!(p.progress().phase_elapsed_ms, 0);
        assert_eq!(p.values(), [50_000]);
    }
}
