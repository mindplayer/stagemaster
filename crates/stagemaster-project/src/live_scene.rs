//! Host scene layers reuse the existing Player and output encoder; no second time evaluator.
use crate::{Document, LiveScenePlayer};
use sha2::{Digest, Sha256};
use stagemaster_playback::Player;

impl Document {
    /// Compile one independently controlled dynamic held scene with sparse ownership.
    /// Different snapshots cannot be mixed merely because vector lengths match.
    /// # Errors
    /// Reject invalid scenes/output layouts or preparation allocation failure.
    pub fn compile_live_scene(
        &self,
        scene_id: &str,
        now_ms: u64,
    ) -> Result<LiveScenePlayer, String> {
        let compiled = self.compile_scene(scene_id)?;
        let identity = Sha256::digest(self.encode()?).into();
        let layout = compiled.output.live_layout(identity)?;
        let mut owned = vec![false; compiled.plan.defaults().len()];
        let view = self.view();
        let scene = view
            .scenes
            .iter()
            .find(|s| s.id == scene_id)
            .ok_or("场景不存在")?;
        for (fixture, key, index, _) in compiled.output.attribute_bindings() {
            owned[index] = scene
                .values
                .iter()
                .any(|v| v.fixture_id == fixture && v.attribute == key && v.mode != "release");
        }
        for effect in &compiled.plan.effects()[0] {
            owned[effect.index] = true;
        }
        let count = owned.len();
        let player =
            Player::try_new(compiled.plan, now_ms).map_err(|_| "无法准备场景播放器缓冲")?;
        Ok(LiveScenePlayer {
            player,
            output: compiled.output,
            layout,
            owned,
            values: vec![None; count],
            claims: vec![false; count],
            asserting: false,
            id: compiled.id,
            revision_id: compiled.revision_id,
        })
    }
}
