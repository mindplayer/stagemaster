#[path = "../examples/capacity_device/progress.rs"]
mod progress;
use stagemaster_package::{Output, Program};
use stagemaster_playback::{Plan, Step};

fn program(waits: &[Option<u64>], repeat: bool) -> Program {
    Program {
        plan: Plan::new(
            vec![0],
            waits
                .iter()
                .map(|wait| Step {
                    target: vec![1],
                    delay_ms: 0,
                    fade_ms: 1000,
                    wait_ms: *wait,
                })
                .collect(),
            repeat,
        )
        .unwrap(),
        output: Output {
            universe: 1,
            mappings: Vec::new(),
        },
        labels: Vec::new(),
    }
}

#[test]
fn repeated_short_steps_use_cycle_position_not_monotonic_step_elapsed() {
    let p = program(&[Some(1000); 3], true);
    assert!(progress::advances(&p, (2, 1700), (2, 700), 47_000));
    assert!(progress::advances(&p, (1, 500), (2, 1000), 44_500));
    assert!(progress::advances(&p, (0, 500), (0, 500), 48_000));
}

#[test]
fn frozen_wrong_step_or_wrong_speed_is_rejected_even_with_same_instance() {
    let p = program(&[Some(1000); 3], true);
    for after in [(2, 1700), (1, 700), (2, 701), (3, 700)] {
        assert!(!progress::advances(&p, (2, 1700), after, 47_000));
    }
    assert!(!progress::advances(&p, (2, 1700), (2, 700), 1000));
    assert!(!progress::advances(&p, (3, 1700), (2, 700), 47_000));
}

#[test]
fn manual_hold_and_transition_into_manual_step_remain_supported() {
    let held = program(&[None], false);
    assert!(progress::advances(&held, (0, 500), (0, 47_500), 47_000));
    let p = program(&[Some(500), None], true);
    assert!(progress::advances(&p, (0, 500), (1, 46_000), 47_000));
}

#[test]
fn finished_nonrepeating_plan_cannot_claim_continuing_playback() {
    let p = program(&[Some(1000); 3], false);
    assert!(!progress::advances(&p, (0, 500), (0, 500), 48_000));
}
