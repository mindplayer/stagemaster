//! Shared integer scene evaluation for sequential playback and direct timeline sampling.
use crate::Plan;

pub(super) fn step(plan: &Plan, index: usize, elapsed_ms: u64, from: &[u16], values: &mut [u16]) {
    let step = &plan.steps[index];
    if elapsed_ms < step.delay_ms {
        values.copy_from_slice(from);
        return;
    }
    let elapsed = elapsed_ms - step.delay_ms;
    values.copy_from_slice(&step.target);
    for effect in &plan.effects[index] {
        values[effect.index] = effect.sample(elapsed, plan.effect_time_offset_ms);
    }
    blend(
        from,
        values,
        &plan.snap_attributes,
        elapsed.saturating_add(plan.entry_fade_offset_ms),
        step.fade_ms,
    );
}

/// Inputs share a validated dimension and sorted discrete mask. Zero duration is a cut.
pub(super) fn blend(from: &[u16], values: &mut [u16], snap: &[u16], elapsed: u64, duration: u64) {
    if elapsed >= duration {
        return;
    }
    let mut snap = snap.iter().peekable();
    for (index, (value, from)) in values.iter_mut().zip(from).enumerate() {
        if snap.peek().is_some_and(|&&i| usize::from(i) == index) {
            snap.next();
            continue;
        }
        let weighted = u64::from(*from) * (duration - elapsed) + u64::from(*value) * elapsed;
        *value =
            u16::try_from((weighted + duration / 2) / duration).expect("bounded interpolation");
    }
}
