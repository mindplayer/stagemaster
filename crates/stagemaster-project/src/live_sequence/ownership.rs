use stagemaster_playback::Observer;

pub(super) struct StepOwnership {
    pub owned: Vec<bool>,
    pub claims: Vec<bool>,
}
pub(super) struct Ownership {
    pub steps: Vec<StepOwnership>,
    pub owned: Vec<bool>,
    pub claims: Vec<bool>,
    pub baseline: Vec<u16>,
    pub winning: Vec<bool>,
}
impl Ownership {
    fn enter(&mut self, step: usize, reassert: bool, from: Option<&mut [u16]>) {
        let target = &self.steps[step];
        if let Some(from) = from {
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
            // Later transitions within the same unsampled advance continue the new
            // source origin; they must not repeatedly jump back to the call's snapshot.
            self.winning[index] = target.owned[index] && (self.winning[index] || asserting);
            self.owned[index] = target.owned[index];
        }
    }
}
impl Observer for Ownership {
    fn activated(&mut self, step: usize, reassert: bool, from: &mut [u16]) {
        self.enter(step, reassert, Some(from));
    }
    fn cycles_skipped(&mut self) {
        // Complete loops have identical metadata transformations. Fold one, even when
        // the sampled step before/after advance is the same or entire loops are omitted.
        for step in 0..self.steps.len() {
            self.enter(step, false, None);
        }
    }
    fn stopped(&mut self) {
        self.owned.fill(false);
        self.claims.fill(false);
    }
}
