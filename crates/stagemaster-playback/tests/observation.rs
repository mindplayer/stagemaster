use stagemaster_playback::{Command, Observer, Plan, Player, Status, Step};

#[derive(Default, Debug, PartialEq, Eq)]
struct Events(Vec<(usize, bool)>, usize, usize);
impl Observer for Events {
    fn activated(&mut self, step: usize, reassert: bool, _: &mut [u16]) {
        self.0.push((step, reassert));
    }
    fn cycles_skipped(&mut self) {
        self.1 += 1;
    }
    fn stopped(&mut self) {
        self.2 += 1;
    }
}
fn step(delay: u64, fade: u64, wait: Option<u64>) -> Step {
    Step {
        target: vec![5000],
        delay_ms: delay,
        fade_ms: fade,
        wait_ms: wait,
    }
}
fn player(steps: Vec<Step>, repeat: bool) -> Player {
    Player::new(Plan::new(vec![1000], steps, repeat).unwrap(), 0)
}
#[test]
fn delay_activation_is_once_and_pause_resume_do_not_reassert() {
    let mut p = player(vec![step(100, 100, None)], false);
    let mut events = Events::default();
    for (command, now) in [
        (Command::Execute(0), 0),
        (Command::Pause, 50),
        (Command::Advance, 500),
        (Command::Resume, 500),
        (Command::Advance, 549),
    ] {
        p.apply_observed(command, now, &mut events).unwrap();
    }
    assert!(events.0.is_empty());
    p.apply_observed(Command::Advance, 550, &mut events)
        .unwrap();
    assert_eq!(events.0, [(0, true)]);
    p.apply_observed(Command::Advance, 600, &mut events)
        .unwrap();
    assert_eq!(p.values(), [3000]);
    p.apply_observed(Command::Pause, 600, &mut events).unwrap();
    p.apply_observed(Command::Resume, 700, &mut events).unwrap();
    assert_eq!(events.0, [(0, true)]);
    p.apply_observed(Command::Execute(0), 700, &mut events)
        .unwrap();
    p.apply_observed(Command::Advance, 800, &mut events)
        .unwrap();
    assert_eq!(events.0, [(0, true), (0, true)]);
    p.apply_observed(Command::Stop, 800, &mut events).unwrap();
    assert_eq!(events.2, 1);
    assert_eq!(p.status(), Status::Idle);
}
#[test]
fn zero_duration_steps_and_same_index_loop_still_emit_authoritative_events() {
    let mut p = player(vec![step(0, 0, Some(0)), step(0, 0, Some(100))], true);
    let mut events = Events::default();
    p.apply_observed(Command::Execute(0), 0, &mut events)
        .unwrap();
    assert_eq!(p.index(), Some(1));
    assert_eq!(events.0, [(0, true), (1, false)]);
    events.0.clear();
    p.apply_observed(Command::Advance, 100, &mut events)
        .unwrap();
    assert_eq!(p.index(), Some(1));
    assert_eq!(events.0, [(0, false), (1, false)]);
    events.0.clear();
    p.apply_observed(Command::Advance, 1_000_000_000_050, &mut events)
        .unwrap();
    assert_eq!(p.index(), Some(1));
    assert_eq!(p.elapsed_ms(), 50);
    assert_eq!(events.1, 1);
    assert_eq!(events.0, [(0, false), (1, false)]);
}
#[test]
fn invalid_commands_do_not_notify_and_manual_next_has_sequential_semantics() {
    let mut p = player(vec![step(100, 0, None), step(0, 0, None)], false);
    let mut events = Events::default();
    p.apply_observed(Command::Execute(0), 0, &mut events)
        .unwrap();
    assert!(
        p.apply_observed(Command::Execute(2), 500, &mut events)
            .is_err()
    );
    assert!(events.0.is_empty());
    assert_eq!(p.elapsed_ms(), 0);
    p.apply_observed(Command::Advance, 100, &mut events)
        .unwrap();
    assert!(p.apply_observed(Command::Advance, 99, &mut events).is_err());
    assert_eq!(events.0, [(0, true)]);
    p.apply_observed(Command::Next, 100, &mut events).unwrap();
    p.apply_observed(Command::Next, 200, &mut events).unwrap();
    assert_eq!(events.0, [(0, true), (1, false)]);
}
#[test]
fn observer_seeds_new_origin_at_delay_end_without_changing_time_or_discrete_rules() {
    struct Seed;
    impl Observer for Seed {
        fn activated(&mut self, _: usize, _: bool, from: &mut [u16]) {
            from.copy_from_slice(&[3000, 7]);
        }
    }
    let plan = Plan::with_snap_attributes(
        vec![1000, 0],
        vec![Step {
            target: vec![5000, 40],
            delay_ms: 100,
            fade_ms: 100,
            wait_ms: None,
        }],
        false,
        vec![vec![]],
        vec![1],
    )
    .unwrap();
    let mut p = Player::new(plan, 0);
    p.apply_observed(Command::Execute(0), 0, &mut Seed).unwrap();
    assert_eq!(p.values(), [1000, 0]);
    p.apply_observed(Command::Advance, 100, &mut Seed).unwrap();
    assert_eq!(p.values(), [3000, 40]);
    p.apply_observed(Command::Advance, 150, &mut Seed).unwrap();
    assert_eq!(p.values(), [4000, 40]);
    assert_eq!(p.elapsed_ms(), 150);
}
