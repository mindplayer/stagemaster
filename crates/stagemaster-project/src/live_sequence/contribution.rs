use super::LiveSequencePlayer;
use stagemaster_engine::live::Error;
use stagemaster_playback::Plan;

impl LiveSequencePlayer {
    /// Immutable prepared plan, for admission budgets and command preflight.
    #[must_use]
    pub fn plan(&self) -> &Plan {
        self.player.plan()
    }

    /// Copy a complete sparse contribution and its latest pending claim time per attribute.
    /// Trusted host use only: serialize read, composition and acknowledgment.
    /// # Errors
    /// Reject either wrong destination shape before changing any destination.
    pub fn copy_contribution(
        &self,
        values: &mut [Option<u16>],
        claim_times: &mut [Option<u64>],
    ) -> Result<(), Error> {
        if values.len() != self.values.len() || claim_times.len() != self.values.len() {
            return Err(Error::Shape);
        }
        for (index, value) in values.iter_mut().enumerate() {
            *value = self.ownership.owned[index].then_some(self.player.values()[index]);
        }
        claim_times.copy_from_slice(&self.ownership.claim_times);
        Ok(())
    }

    /// Clear claims only after a complete successful composition, never after a failed publish.
    pub fn acknowledge_contribution(&mut self) {
        self.ownership.claims.fill(false);
        self.ownership.claim_times.fill(None);
    }
}
