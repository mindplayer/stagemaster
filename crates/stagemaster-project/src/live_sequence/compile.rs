use super::{LiveSequencePlayer, Ownership, StepOwnership};
use crate::Document;
use sha2::{Digest, Sha256};
use stagemaster_playback::Player;

impl Document {
    /// Prepare a sparse list source; reuse the existing plan, targets and effect evaluator.
    /// # Errors
    /// Reject invalid lists, unsupported layouts or failed player preparation.
    pub fn compile_live_sequence(
        &self,
        sequence_id: &str,
        now_ms: u64,
    ) -> Result<LiveSequencePlayer, String> {
        let compiled = self.compile_sequence(sequence_id)?;
        let layout = compiled
            .output
            .live_layout(Sha256::digest(self.encode()?).into())?;
        let count = layout.attributes().len();
        let view = self.view();
        let list = view
            .sequences
            .iter()
            .find(|s| s.id == sequence_id)
            .ok_or("场景列表不存在")?;
        let mut tracked = vec![false; count];
        let mut ownership = Vec::with_capacity(list.steps.len());
        for (index, step) in list.steps.iter().enumerate() {
            if list.tracking == "isolated" {
                tracked.fill(false);
            }
            let scene = view
                .scenes
                .iter()
                .find(|s| s.id == step.scene_id)
                .ok_or("步骤引用的场景不存在")?;
            let mut claims = vec![false; count];
            for (fixture, key, attr, _) in compiled.output.attribute_bindings() {
                if let Some(value) = scene
                    .values
                    .iter()
                    .find(|v| v.fixture_id == fixture && v.attribute == key)
                {
                    tracked[attr] = value.mode != "release";
                    claims[attr] = tracked[attr];
                }
            }
            let mut owned = tracked.clone();
            // Effects are local to each compiled step; do not silently add shape tracking.
            for effect in &compiled.plan.effects()[index] {
                owned[effect.index] = true;
                claims[effect.index] = true;
            }
            ownership.push(StepOwnership { owned, claims });
        }
        let player =
            Player::try_new(compiled.plan, now_ms).map_err(|_| "无法准备列表播放器缓冲")?;
        Ok(LiveSequencePlayer {
            player,
            output: compiled.output,
            layout,
            ownership: Ownership {
                steps: ownership,
                owned: vec![false; count],
                claims: vec![false; count],
                baseline: vec![0; count],
                winning: vec![false; count],
            },
            values: vec![None; count],
            winners: vec![None; count],
            id: compiled.id,
            revision_id: compiled.revision_id,
            steps: compiled.steps,
        })
    }
}
