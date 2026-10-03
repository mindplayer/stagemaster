//! Independent timeline arithmetic for control acceptance, including step-local clocks.
use stagemaster_package::Program;

pub fn advances(
    program: &Program,
    before: (usize, u64),
    after: (usize, u64),
    delta_ms: u64,
) -> bool {
    let steps = program.plan.steps();
    if before.0 >= steps.len() || after.0 >= steps.len() || delta_ms < 44_500 {
        return false;
    }
    let durations: Vec<_> = steps
        .iter()
        .map(|s| s.wait_ms.map(|wait| s.delay_ms + s.fade_ms + wait))
        .collect();
    let mut index = before.0;
    let Some(mut remaining) = before.1.checked_add(delta_ms) else {
        return false;
    };
    // An automatic repeating plan may wrap many times during disconnection.
    if program.plan.repeat() && durations.iter().all(Option::is_some) {
        let cycle: u64 = durations.iter().flatten().sum();
        if cycle == 0 {
            return false;
        }
        remaining %= cycle;
    }
    for _ in 0..=steps.len() {
        match durations[index] {
            Some(duration) if remaining >= duration => {
                remaining -= duration;
                index += 1;
                if index == steps.len() {
                    if !program.plan.repeat() {
                        return false;
                    }
                    index = 0;
                }
            }
            _ => return after == (index, remaining),
        }
    }
    false
}
