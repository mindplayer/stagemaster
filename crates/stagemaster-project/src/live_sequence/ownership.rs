use stagemaster_playback::{Activation, Observer};

pub(super) struct StepOwnership {
    pub owned: Vec<bool>,
    pub claims: Vec<bool>,
    pub activation_offset_ms: u64,
}
pub(super) struct Ownership {
    pub use_live_baseline: bool,
    pub steps: Vec<StepOwnership>,
    pub owned: Vec<bool>,
    pub claims: Vec<bool>,
    pub claim_times: Vec<Option<u64>>,
    pub baseline: Vec<u16>,
    pub winning: Vec<bool>,
}
impl Ownership {
    fn enter(&mut self, step: usize, reassert: bool, at_ms: u64, from: Option<&mut [u16]>) {
        let target = &self.steps[step];
        if let Some(from) = from.filter(|_| self.use_live_baseline) {
            for (index, value) in from.iter_mut().enumerate() {
                let asserting = reassert || target.claims[index];
                if target.owned[index]
                    && (!self.owned[index] || (asserting && !self.winning[index]))
                {
                    *value = self.baseline[index];
                }
            }
        }
        for index in 0..self.owned.len() {
            let asserting =
                target.owned[index] && (reassert || target.claims[index] || !self.owned[index]);
            self.claims[index] = target.owned[index] && (self.claims[index] || asserting);
            if !target.owned[index] {
                self.claim_times[index] = None;
            } else if asserting {
                self.claim_times[index] = Some(at_ms);
            }
            // Later transitions within the same unsampled advance continue the new
            // source origin; they must not repeatedly jump back to the call's snapshot.
            self.winning[index] = target.owned[index] && (self.winning[index] || asserting);
            self.owned[index] = target.owned[index];
        }
    }
}
impl Observer for Ownership {
    fn activated_at(&mut self, event: Activation, from: &mut [u16]) {
        self.enter(event.step, event.reassert, event.at_ms, Some(from));
    }
    fn cycles_skipped_at(&mut self, last_cycle_started_ms: u64) {
        // Complete loops have identical metadata transformations. Fold one, even when
        // the sampled step before/after advance is the same or entire loops are omitted.
        for step in 0..self.steps.len() {
            let at_ms = last_cycle_started_ms + self.steps[step].activation_offset_ms;
            self.enter(step, false, at_ms, None);
        }
    }
    fn stopped(&mut self) {
        self.owned.fill(false);
        self.claims.fill(false);
        self.claim_times.fill(None);
    }
}
