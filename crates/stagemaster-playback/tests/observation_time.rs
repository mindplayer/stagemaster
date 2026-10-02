use stagemaster_playback::{Activation, Command, Observer, Plan, Player, Step};

#[derive(Default)]
struct Times(Vec<Activation>, Vec<u64>);
impl Observer for Times {
    fn activated_at(&mut self, event: Activation, _: &mut [u16]) {
        self.0.push(event);
    }
    fn cycles_skipped_at(&mut self, start: u64) {
        self.1.push(start);
    }
}
fn player(repeat: bool) -> Player {
    let steps = [(10, 10), (20, 60)]
        .into_iter()
        .map(|(delay, wait)| Step {
            target: vec![5000],
            delay_ms: delay,
            fade_ms: 0,
            wait_ms: Some(wait),
        })
        .collect();
    Player::new(Plan::new(vec![0], steps, repeat).unwrap(), 1000)
}
#[test]
fn delayed_sampling_reports_actual_boundaries_and_pause_adjusts_clock() {
    let mut p = player(false);
    let mut t = Times::default();
    for (c, now) in [
        (Command::Execute(0), 1000),
        (Command::Pause, 1005),
        (Command::Resume, 2005),
        (Command::Advance, 2099),
    ] {
        p.apply_observed(c, now, &mut t).unwrap();
    }
    assert_eq!(
        t.0.iter().map(|e| e.at_ms).collect::<Vec<_>>(),
        [2010, 2040]
    );
    assert!(t.0[0].reassert);
    assert!(!t.0[1].reassert);
    assert!(t.1.is_empty());
}
#[test]
fn skipped_cycles_identify_last_complete_cycle_even_at_clock_limit() {
    let mut p = player(true);
    let mut t = Times::default();
    p.apply_observed(Command::Execute(0), 1000, &mut t).unwrap();
    p.apply_observed(Command::Advance, u64::MAX, &mut t)
        .unwrap();
    let remainder = (u64::MAX - 1000) % 100;
    let current = u64::MAX - remainder;
    assert_eq!(t.1, [current - 100]);
    assert_eq!(t.0.last().unwrap().at_ms, current + 10);
    assert!(t.0.len() <= 4);
    let before = t.0.clone();
    assert!(
        p.apply_observed(Command::Advance, u64::MAX - 1, &mut t)
            .is_err()
    );
    assert_eq!(t.0, before);
}
