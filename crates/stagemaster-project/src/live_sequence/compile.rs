use super::{LiveSequencePlayer, Ownership, StepOwnership};
use crate::{CompiledSequence, Document, PackageSelection};
use sha2::{Digest, Sha256};
use stagemaster_playback::Player;

impl Document {
    /// Prepare a sparse list source using the existing plan and effect evaluator.
    /// # Errors
    /// Reject invalid lists, unsupported layouts or failed player preparation.
    pub fn compile_live_sequence(
        &self,
        sequence_id: &str,
        now_ms: u64,
    ) -> Result<LiveSequencePlayer, String> {
        self.compile_live_source(
            &PackageSelection::Sequence {
                id: sequence_id.into(),
            },
            now_ms,
        )
    }

    /// Prepare a held scene or list for the same contribution interface.
    /// # Errors
    /// Reject invalid selection, output mapping or preparation failure.
    pub fn compile_live_source(
        &self,
        selection: &PackageSelection,
        now_ms: u64,
    ) -> Result<LiveSequencePlayer, String> {
        let (compiled, tracking, scenes) = match selection {
            PackageSelection::Scene { id } => (self.compile_scene(id)?, false, vec![id.clone()]),
            PackageSelection::Sequence { id } => {
                let view = self.view();
                let list = view
                    .sequences
                    .iter()
                    .find(|s| s.id == *id)
                    .ok_or("场景列表不存在")?;
                (
                    self.compile_sequence(id)?,
                    list.tracking != "isolated",
                    list.steps.iter().map(|s| s.scene_id.clone()).collect(),
                )
            }
        };
        self.prepare_contribution(compiled, tracking, &scenes, now_ms)
    }

    fn prepare_contribution(
        &self,
        compiled: CompiledSequence,
        tracking: bool,
        scenes: &[String],
        now_ms: u64,
    ) -> Result<LiveSequencePlayer, String> {
        let layout = compiled
            .output
            .live_layout(Sha256::digest(self.encode()?).into())?;
        let count = layout.attributes().len();
        let view = self.view();
        let mut tracked = vec![false; count];
        let mut ownership = Vec::with_capacity(compiled.steps.len());
        let mut step_start_ms = 0;
        for (index, scene_id) in scenes.iter().enumerate() {
            if !tracking {
                tracked.fill(false);
            }
            let scene = view
                .scenes
                .iter()
                .find(|s| s.id == *scene_id)
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
            for effect in &compiled.plan.effects()[index] {
                owned[effect.index] = true;
                claims[effect.index] = true;
            }
            let timing = &compiled.plan.steps()[index];
            ownership.push(StepOwnership {
                owned,
                claims,
                activation_offset_ms: step_start_ms + timing.delay_ms,
            });
            // These offsets are used only for fully automatic cycles. Manual plans cannot skip cycles.
            step_start_ms += timing.delay_ms + timing.fade_ms + timing.wait_ms.unwrap_or(0);
        }
        let player = Player::try_new(compiled.plan, now_ms).map_err(|_| "无法准备播放器缓冲")?;
        Ok(LiveSequencePlayer {
            player,
            output: compiled.output,
            layout,
            ownership: Ownership {
                use_live_baseline: true,
                steps: ownership,
                owned: vec![false; count],
                claims: vec![false; count],
                claim_times: vec![None; count],
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
